// NOTE: This code for now might need `vim.fn.serverstart('/tmp/nvim.sock')` on `init.lua` entrypoint
use nvim_rs::{create::tokio::new_path, rpc::handler::Dummy, Value};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = match env::var("NVIM_LISTEN_ADDRESS") {
        Ok(path) => {
            println!("Found NVIM_LISTEN_ADDRESS: {}", path);
            path
        }
        Err(_) => {
            let possible_paths = vec![
                "/tmp/nvim.sock".to_string(),
                "/tmp/nvim".to_string(),
                format!("/tmp/nvim-{}/0", env::var("USER").unwrap_or_else(|_| "user".to_string())),
            ];
            
            let mut found_path = None;
            for path in &possible_paths {
                if Path::new(path).exists() {
                    found_path = Some(path.clone());
                    break;
                }
            }
            
            match found_path {
                Some(path) => {
                    println!("Found Neovim socket at: {}", path);
                    path
                }
                None => {
                    eprintln!("No Neovim instance found. Please:");
                    eprintln!("1. Start Neovim with: nvim --listen /tmp/nvim.sock");
                    eprintln!("2. Or run this from within a Neovim terminal");
                    eprintln!("3. Or set NVIM_LISTEN_ADDRESS environment variable");
                    std::process::exit(1);
                }
            }
        }
    };
    
    let (nvim, io_handler) = new_path(&socket_path, Dummy::new()).await?;
    
    tokio::spawn(async move {
        if let Err(err) = io_handler.await {
            eprintln!("nvim IO error: {:?}", err);
        }
    });
    
    println!("\n=== Test 1: :echo bufnr('%') ===");
    let out = nvim.command_output("echo bufnr('%')").await?;
    println!("Neovim says: {}", out);
    
    println!("\n=== Test 2: :lua print(vim.api.nvim_get_current_buf()) ===");
    nvim.command("lua print('From Rust: Current buffer is ' .. vim.api.nvim_get_current_buf())").await?;
    println!("Lua print command sent (check Neovim messages with :messages)");
    
    println!("\n=== Test 3: Direct API call vim.api.nvim_get_current_buf() ===");
    let current_buf = nvim.get_current_buf().await?;
    let buf_number = current_buf.get_number().await?;
    println!("Current buffer number (via direct API): {}", buf_number);
    
    println!("\n=== Test 4: Execute Lua and capture return value ===");
    let lua_result = nvim.exec_lua("return vim.api.nvim_get_current_buf()", vec![]).await?;
    println!("Lua return value: {:?}", lua_result);
    
    match lua_result {
        Value::Integer(int_val) => println!("Buffer number as Rust uint: {:?}", int_val.as_u64().unwrap()),
        _ => println!("Could not extract u64 from Lua result: {:?}", lua_result),
    }
    
    println!("\n=== Bonus: Get current buffer name ===");
    let buf_name = nvim.command_output("echo expand('%:p')").await?;
    println!("Current buffer path: {}", buf_name);
    
    Ok(())
}
