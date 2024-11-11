// This file is ran at the start of the enclave to "encumber" the account and make sure this TEE ai agent is the only one that has access to it

use rand::{thread_rng, Rng};
use std::{ffi::OsStr, sync::Arc, time::Duration};

use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AccountDetails {
    pub x_username: String,
    pub x_password: String,
    pub email: String,
    pub email_password: String,
}

#[derive(Debug, Clone)]
pub struct FullAccountDetails {
    pub x_account: XAccountDetails,
    pub email: String,
    pub email_password: String,
}

#[derive(Debug, Clone)]
pub struct XAccountDetails {
    pub x_email: String,
    pub x_username: String,
    pub x_password: String,
    pub x_consumer_key: String,
    pub x_consumer_secret: String,
    pub x_access_token: String,
    pub x_access_token_secret: String,
}

pub fn encumber(account_details: AccountDetails) -> FullAccountDetails {
    let x_account = encumber_twitter_account(
        &account_details.x_username,
        &account_details.x_password,
        &account_details.email,
    );

    let email_password =
        change_mail_password(&account_details.email, &account_details.email_password);

    FullAccountDetails {
        x_account,
        email: account_details.email,
        email_password,
    }
}

fn get_browser() -> Browser {
    let options = LaunchOptionsBuilder::default()
        .sandbox(false)
        .path(Some("./chrome-linux/chrome".into()))
        .user_data_dir(Some("./chrome_user_data".into()))
        .args(
            [
                OsStr::new("--start-maximized"),
                OsStr::new("--disable-dev-shm-usage"),
                OsStr::new("--window-size=1920,1080"),
            ]
            .into(),
        )
        .build()
        .expect("failed building chrome options");
    Browser::new(options).expect("Failed to build chrome browser")
}
fn encumber_twitter_account(x_username: &str, x_password: &str, x_email: &str) -> XAccountDetails {
    let browser = get_browser();
    let tab = browser.new_tab().expect("Failed to create tab");

    tab.navigate_to("https://twitter.com/i/flow/login").unwrap();
    let username = tab
        .wait_for_element("input[autocomplete=\"username\"]")
        .unwrap();
    username.type_into(x_username).unwrap();
    tab.press_key("Enter").unwrap();

    let password = if let Ok(email) = tab.wait_for_element("input[autocomplete=\"on\"]") {
        email.type_into(x_email).unwrap();
        tab.press_key("Enter").unwrap();
        tab.wait_for_element("input[autocomplete=\"current-password\"]")
            .unwrap()
    } else {
        tab.wait_for_element("input[name=\"password\"]").unwrap()
    };

    password.type_into(x_password).unwrap();
    tab.press_key("Enter").unwrap();

    // we should sleep here a bit
    std::thread::sleep(Duration::from_secs(3));

    let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
        regenerate_x_tokens(tab.clone());

    let random_password = generate_random_password(10, 1, 1, 1);

    tab.navigate_to("https://x.com/settings/password").unwrap();
    let current_password = tab
        .wait_for_element("input[name=\"current_password\"]")
        .unwrap();
    current_password.type_into(x_password).unwrap();

    let new_password = tab
        .wait_for_element("input[name=\"new_password\"]")
        .unwrap();
    new_password.type_into(&random_password).unwrap();

    let confirm_password = tab
        .wait_for_element("input[name=\"password_confirmation\"]")
        .unwrap();
    confirm_password.type_into(&random_password).unwrap();

    let button = tab
        .find_element("button[data-testid=\"settingsDetailSave\"]")
        .unwrap();

    button.click().unwrap();
    std::thread::sleep(Duration::from_secs(3));

    XAccountDetails {
        x_email: x_email.into(),
        x_username: x_username.into(),
        x_password: x_password.into(),
        x_consumer_key,
        x_consumer_secret,
        x_access_token,
        x_access_token_secret,
    }
}

fn change_mail_password(email_string: &str, password_string: &str) -> String {
    let browser = get_browser();
    let tab = browser.new_tab().expect("Failed to create tab");

    tab.navigate_to("https://cock.li/login").unwrap();

    let email = tab.wait_for_element("input[name=\"email\"").unwrap();
    email.type_into(email_string).unwrap();

    let password = tab.wait_for_element("input[name=\"password\"").unwrap();
    password.type_into(password_string).unwrap();

    tab.press_key("Enter").unwrap();
    std::thread::sleep(Duration::from_secs(5));

    tab.navigate_to("https://cock.li/user/changepass").unwrap();

    let random_pass = generate_random_password(10, 1, 1, 1);

    let current_pass = tab
        .wait_for_element("input[name=\"current_password\"]")
        .unwrap();
    current_pass.type_into(password_string).unwrap();

    let new_pass = tab.wait_for_element("input[name=\"password\"]").unwrap();
    new_pass.type_into(&random_pass).unwrap();

    let pass_confirm = tab
        .wait_for_element("input[name=\"password_confirmation\"]")
        .unwrap();
    pass_confirm.type_into(&random_pass).unwrap();

    tab.press_key("Enter").unwrap();
    std::thread::sleep(Duration::from_secs(5));
    random_pass
}

fn regenerate_x_tokens(tab: Arc<Tab>) -> (String, String, String, String) {
    tab.navigate_to("https://developer.x.com/en/portal/projects-and-apps")
        .unwrap();

    let keys_button = tab.wait_for_element("img[alt=\"keys\"").unwrap();
    keys_button.click().unwrap();

    // Regenerate the Consumer Keys
    // If the other keys are revoke this will be the only button on screen with this class
    let consumer_regenerate = tab
        .wait_for_element("button[class=\"Button Button--primary\"]")
        .unwrap();
    consumer_regenerate.click().unwrap();

    tab.wait_for_element("button[data-testid=\"confirmation-dev-portal-dialog-action-button\"")
        .unwrap()
        .click()
        .unwrap();

    let api_key = tab
        .wait_for_xpath("/html/body/div[5]/div/div/div[1]/div[2]/div[2]/div[2]/p")
        .unwrap();
    let consumer_key = api_key.get_inner_text().unwrap();

    let api_secret = tab
        .wait_for_xpath("/html/body/div[5]/div/div/div[1]/div[2]/div[3]/div[2]/p")
        .unwrap();

    let consumer_secret = api_secret.get_inner_text().unwrap();

    tab.wait_for_xpath("/html/body/div[5]/div/div/div[2]/div/button")
        .unwrap()
        .click()
        .unwrap();

    tab.wait_for_xpath(
        "/html/body/div[1]/div/div/div[2]/div[2]/div[2]/div[1]/div/div[3]/div[2]/div/button",
    )
    .unwrap()
    .click()
    .unwrap();

    // todo: This will fail if they already have a token generated here, set up a flow to handle this case as well
    let access_token = tab
        .wait_for_xpath("/html/body/div[5]/div/div/div[1]/div[2]/div[2]/div[2]/p")
        .unwrap()
        .get_inner_text()
        .unwrap();

    let access_token_secret = tab
        .wait_for_xpath("/html/body/div[5]/div/div/div[1]/div[2]/div[3]/div[2]/p")
        .unwrap()
        .get_inner_text()
        .unwrap();

    tab.wait_for_xpath("/html/body/div[5]/div/div/div[2]/div/button")
        .unwrap()
        .click()
        .unwrap();

    std::thread::sleep(Duration::from_secs(3));

    (
        consumer_key,
        consumer_secret,
        access_token,
        access_token_secret,
    )
}

impl From<&Config> for AccountDetails {
    fn from(value: &Config) -> Self {
        AccountDetails {
            x_username: value.x_username.clone(),
            x_password: value.x_password.clone(),
            email: value.email.clone(),
            email_password: value.email_password.clone(),
        }
    }
}

fn generate_random_password(
    length: usize,
    min_uppercase: usize,
    min_numbers: usize,
    min_special: usize,
) -> String {
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const NUMBERS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!@#$%^&*";

    let mut rng = thread_rng();
    let mut password = Vec::with_capacity(length);

    // Add minimum required characters
    for _ in 0..min_uppercase {
        password.push(UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char);
    }
    for _ in 0..min_numbers {
        password.push(NUMBERS[rng.gen_range(0..NUMBERS.len())] as char);
    }
    for _ in 0..min_special {
        password.push(SPECIAL[rng.gen_range(0..SPECIAL.len())] as char);
    }

    // Fill the rest with lowercase letters
    let remaining = length - (min_uppercase + min_numbers + min_special);
    for _ in 0..remaining {
        password.push(LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char);
    }

    // Shuffle the password
    for i in (1..password.len()).rev() {
        let j = rng.gen_range(0..=i);
        password.swap(i, j);
    }

    password.into_iter().collect()
}

#[test]
fn test_random_pass() {
    for _ in 0..100 {
        let pass = generate_random_password(10, 2, 2, 2);

        println!("{pass}");
    }
}

#[test]
fn test() {
    let browser = get_browser();
    let tab = browser.new_tab().unwrap();

    tab.navigate_to("https://google.com").unwrap();
}
