// Based on AndroidNavigatorInterface from ruffle-android
// https://github.com/ruffle-rs/ruffle-android/blob/93ee5149695400dca4bc5d4629e3bf5ab2145162/src/navigator.rs

use std::fs::File;
use std::path::Path;
use url::Url;

use ruffle_frontend_utils::backends::navigator::NavigatorInterface;

#[derive(Clone)]
pub struct ConsoleNavigatorInterface;

// TODO: Prompt the user for these things!
impl NavigatorInterface for ConsoleNavigatorInterface {
    fn navigate_to_website(&self, _url: Url) {
        unimplemented!()
    }

    async fn open_file(&self, path: &Path) -> std::io::Result<File> {
        #[cfg(not(target_os = "vita"))]
        return File::open(path);
        #[cfg(target_os = "vita")]
        return File::open(format!("{}{}", "ux0:", path.to_string_lossy()));
    }

    async fn confirm_socket(&self, _host: &str, _port: u16) -> bool {
        true
    }
}
