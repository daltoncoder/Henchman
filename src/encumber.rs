// This file is ran at the start of the enclave to "encumber" the account and make sure this TEE ai agent is the only one that has access to it

use rand::{thread_rng, Rng};
use std::{ffi::OsStr, time::Duration};

use headless_chrome::{Browser, LaunchOptionsBuilder};

#[derive(Clone)]
pub struct AccountDetails {
    pub x_email: String,
    pub x_username: String,
    pub x_password: String,
    pub email: String,
    pub email_password: String,
}

pub async fn encumber(account_details: AccountDetails) -> AccountDetails {
    let mut new_details = account_details.clone();

    new_details.x_password = change_twitter_password(
        &account_details.x_username,
        &account_details.x_password,
        &account_details.x_email,
    );

    new_details.email_password =
        change_mail_password(&account_details.email, &account_details.email_password);

    new_details
}

fn get_browser() -> Browser {
    let options = LaunchOptionsBuilder::default()
        .sandbox(false)
        .args(
            [
                OsStr::new("--start-maximized"),
                OsStr::new("--disable-dev-shm-usage"),
            ]
            .into(),
        )
        .build()
        .expect("failed building chrome options");
    Browser::new(options).expect("Failed to build chrome browser")
}
fn change_twitter_password(x_username: &str, x_password: &str, x_email: &str) -> String {
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
    std::thread::sleep(Duration::from_secs(5));
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
    random_password
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
