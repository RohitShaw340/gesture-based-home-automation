use gesture_ease::config::Config;
use gesture_ease::App;

fn main() {
    let config = Config::open("config.toml".into()).unwrap();

    let app = App::new(config).unwrap();

    app.run().unwrap();
}
