use errors::SendgridResult;

use mail::Mail;

use std::io::Read;
use std::collections::HashMap;

use reqwest::header::{Authorization, Bearer, ContentType, Headers, UserAgent};
use reqwest::Client;

use base64;
use json::JsonValue;

static API_URL: &'static str = "https://api.sendgrid.com/v3/mail/send";

fn hashmap_to_json(map: &HashMap<String, String>) -> JsonValue {
    let mut object = JsonValue::new_object();
    for (k, v) in map.iter() {
        object[k] = v.clone().into();
    }
    object
}

/// This is the struct that allows you to authenticate to the SendGrid API.
/// It's only field is the API key which allows you to send messages.
pub struct SGClient {
    api_key: String,
}

// Use the URL form encoder to properly generate the body used in the mail send request.
fn make_post_body(mail_info: &Mail) -> SendgridResult<String> {

    let mut jsonmail = object!{
        "personalizations" => array![
            object!{
                "to" => array![]
            }
        ],
        "from" => object!{
            "email" => mail_info.from.clone()
        },
        "subject" => mail_info.subject.clone(),
        "content" => array![]
    };

    if mail_info.from_name.len() > 0 {
        jsonmail["from"]["name"] = mail_info.from_name.clone().into();
    }

    if mail_info.reply_to.len() > 0 {
        jsonmail["reply_to"] = object!{
            "email" => mail_info.reply_to.clone()
        };
    }

    if mail_info.headers.len() > 0 {
        jsonmail["headers"] = hashmap_to_json(&mail_info.headers);
    }

    for (i, to) in mail_info.to.iter().enumerate() {
        let mut object = JsonValue::new_object();
        object["email"] = to.clone().into();
        if mail_info.to_names.len() > i {
            object["name"] = mail_info.to_names[i].clone().into();
        }
        jsonmail["personalizations"][0]["to"].push(object).unwrap();
    }

    if mail_info.cc.len() > 0 {
        jsonmail["personalizations"][0]["cc"] = array![];
        for cc in mail_info.cc.iter() {
            jsonmail["personalizations"][0]["cc"].push(object!{
                "email" => cc.clone()
            }).unwrap();
        }
    }

    if mail_info.bcc.len() > 0 {
        jsonmail["personalizations"][0]["bcc"] = array![];
        for bcc in mail_info.bcc.iter() {
            jsonmail["personalizations"][0]["bcc"].push(object! {
                "email" => bcc.clone()
            }).unwrap();
        }
    }

    if mail_info.attachments.len() > 0 {
        jsonmail["attachments"] = array![];
        for (attachment, contents) in &mail_info.attachments {
            jsonmail["attachments"].push(object!{
                "content" => base64::encode(contents),
                "filename" => attachment.clone()
            }).unwrap();
        }
    }

    if mail_info.text.len() > 0 {
        jsonmail["content"].push(object! {
            "type" => "text/plain",
            "value" => mail_info.text.clone()
        }).unwrap();
    }
    if mail_info.html.len() > 0 {
        jsonmail["content"].push(object! {
            "type" => "text/html",
            "value" => mail_info.html.clone()
        }).unwrap();
    }

    for (mimetype, value) in &mail_info.content {
        jsonmail["content"].push(object!{
            "type" => mimetype.clone(),
            "value" => value.clone()
        }).unwrap();
    }

    // unused fields. Necessary?
    //encoder.append_pair("date", &mail_info.date);
    //encoder.append_pair("x-smtpapi", &mail_info.x_smtpapi);

    let body = jsonmail.dump();
    Ok(body)
}

impl SGClient {
    /// Makes a new SendGrid cient with the specified API key.
    pub fn new(key: String) -> SGClient {
        SGClient { api_key: key }
    }

    /// Sends a messages through the SendGrid API. It takes a Mail struct as an
    /// argument. It returns the string response from the API as JSON.
    /// It sets the Content-Type to be application/x-www-form-urlencoded.
    pub fn send(self, mail_info: &Mail) -> SendgridResult<String> {
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer {
            token: self.api_key.to_owned(),
        }));
        headers.set(ContentType::json());
        headers.set(UserAgent::new("sendgrid-rs"));

        let post_body = make_post_body(mail_info)?;
        let mut res = client
            .post(API_URL)
            .headers(headers)
            .body(post_body)
            .send()?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        Ok(body)
    }
}

#[test]
fn basic_message_body() {
    let mut m = Mail::new();
    m.add_to("test@example.com");
    m.add_from("me@example.com");
    m.add_subject("Test");
    m.add_text("It works");

    let body = make_post_body(m);
    let want = "to%5B%5D=test%40example.com&from=me%40example.com&subject=Test&\
                html=&text=It+works&fromname=&replyto=&date=&headers=%7B%7D&x-smtpapi=";
    assert_eq!(body.unwrap(), want);
}

#[test]
fn test_proper_key() {
    let want = "files[test.jpg]";
    let got = make_form_key("files", "test.jpg");
    assert_eq!(want, got);
}
