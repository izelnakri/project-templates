use std::{error::Error, future::pending};
use zbus::{connection, interface, proxy, fdo};
use zbus::object_server::{SignalEmitter};
use event_listener::{Event, Listener};

struct Greeter {
    name: String,
    done: Event,
}

#[interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }

    // Rude!
    async fn go_away(
        &self,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        emitter.greeted_everyone().await?;
        self.done.notify(1);

        Ok(())
    }

    /// A "GreeterName" property.
    #[zbus(property)]
    async fn greeter_name(&self) -> &str {
        &self.name
    }

    /// A setter for the "GreeterName" property.
    ///
    /// Additionally, a `greeter_name_changed` method has been generated for you if you need to
    /// notify listeners that "GreeterName" was updated. It will be automatically called when
    /// using this setter.
    #[zbus(property)]
    async fn set_greeter_name(&mut self, name: String) {
        self.name = name;
    }

    /// A signal; the implementation is provided by the macro.
    #[zbus(signal)]
    async fn greeted_everyone(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let greeter = Greeter {
        name: "GreeterName".to_string(),
        done: event_listener::Event::new(),
    };
    let done_listener = greeter.done.listen();
    let connection = connection::Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)? // This replaces existing definition
        .build()
        .await?;

    done_listener.wait();

    // Let's emit the signal again, just for the fun of it.
    connection
        .object_server()
        .interface("/org/zbus/MyGreeter") // or .at('', greeter)? This is useful to register interfaces dynamically
        .await?
        .greeted_everyone()
        .await?;

    pending::<()>().await;
    Ok(())
}
