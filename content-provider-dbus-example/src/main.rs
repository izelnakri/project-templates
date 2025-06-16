use std::{error::Error, future::pending};
use zbus::{connection, interface, proxy};

struct Greeter {
    count: u64
}

#[interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    // Can be `async` as well.
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called {} times.", name, self.count)
    }

    // NOTE: make also an async method that calls and returns data from async op, then test it to
    // see how it works
}

#[proxy(
    interface = "org.zbus.MyGreeter1",
    default_service = "org.zbus.MyGreeter",
    default_path = "/org/zbus/MyGreeter"
)]
trait Greeter {
    async fn say_hello(&self, name: &str) -> Result<String, zbus::Error>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let greeter = Greeter { count: 0 };
    let connection = connection::Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)?
        .build()
        .await?;

    connection
        .object_server()
        .at("/org/zbus/MyGreeter")
        .await?;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::{connection};
    use zbus::Connection;
    use tokio;

    async fn spawn_service() -> zbus::Result<Connection> {
        let greeter = Greeter { count: 0 };
        connection::Builder::session()?
            .name("org.zbus.MyGreeter")?
            .serve_at("/org/zbus/MyGreeter", greeter)?
            .build()
            .await
    }

    #[tokio::test]
    async fn test_roundtrip() -> zbus::Result<()> {
        let connection = spawn_service().await?; // or // let connection = Connection::session().await?;

        let proxy = GreeterProxy::new(&connection).await?;
        let content = proxy.say_hello("Izel").await?;
        // let proxy = Proxy::new(
        //     &service_conn,
        //     "org.zbus.MyGreeter",
        //     "/org/zbus/MyGreeter",
        //     "org.zbus.MyGreeter1",
        // )
        // .await?;
        // let msg = proxy.call_method("SayHello", &("Izel")).await?;
        // let content: String = msg.body().deserialize()?;

        assert_eq!(content, "Hello Izel! I have been called 1 times.");

        Ok(())
    }
}

// # Notifying property changes

// let mut iface = iface_ref.get_mut().await;
// iface.name = String::from("👋");
// iface.greeter_name_changed(iface_ref.signal_emitter()).await?;
