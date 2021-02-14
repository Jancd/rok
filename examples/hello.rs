use rok::{App, Request};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/ping", |_| async { "pong" });

    app.get("/hello/:echo", |req: Request| async move {
        let echo: String = req.get_param("echo").unwrap_or_default();

        format!("Hello, {}!", echo)
    });

    let port = ":8080";
    println!("start web server,listening {}", port);
    app.run("127.0.0.1".to_string() + port).await.unwrap();
}
