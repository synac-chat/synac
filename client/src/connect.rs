use *;
use common::Packet;
use openssl::ssl::{SslConnector, SSL_VERIFY_PEER};
use rusqlite::Connection as SqlConnection;

use frontend;

pub fn connect(
    db: &SqlConnection,
    ip: &str,
    nick: &str,
    screen: &frontend::Screen,
    ssl: &SslConnector
) -> Option<Session> {
    // See https://github.com/rust-lang/rust/issues/35853
    macro_rules! println {
        () => { screen.log(String::new()); };
        ($($arg:expr),*) => { screen.log(format!($($arg),*)); };
    }
    macro_rules! readline {
        ($break:block) => {
            match screen.readline() {
                Ok(ok) => ok,
                Err(_) => $break
            }
        }
    }
    macro_rules! readpass {
        ($break:block) => {
            match screen.readpass() {
                Ok(ok) => ok,
                Err(_) => $break
            }
        }
    }

    let addr = match parse_ip(ip) {
        Some(some) => some,
        None => {
            println!("Could not parse IP");
            return None;
        }
    };

    let mut stmt = db.prepare("SELECT key, token FROM servers WHERE ip = ?").unwrap();
    let mut rows = stmt.query(&[&addr.to_string()]).unwrap();

    let public_key: String;
    let mut token: Option<String> = None;
    if let Some(row) = rows.next() {
        let row = row.unwrap();
        public_key = row.get(0);
        token = row.get(1);
    } else {
        println!("To securely connect, data from the server (\"public key\") is needed.");
        println!("You can obtain the \"public key\" from the server owner.");
        println!("Enter the key here:");
        public_key = readline!({ return None; });

        db.execute(
            "INSERT INTO servers (ip, key) VALUES (?, ?)",
            &[&addr.to_string(), &public_key]
        ).unwrap();
    }
    let stream = match TcpStream::connect(addr) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Could not connect!");
            println!("{}", err);
            return None;
        }
    };
    let mut stream = {
        let mut config = ssl.configure().expect("Failed to configure SSL connector");
        config.ssl_mut().set_verify_callback(SSL_VERIFY_PEER, move |_, cert| {
            match cert.current_cert() {
                Some(cert) => match cert.public_key() {
                    Ok(pkey) => match pkey.public_key_to_pem() {
                        Ok(pem) => {
                            let digest = openssl::sha::sha256(&pem);
                            let mut digest_str = String::with_capacity(64);
                            for byte in &digest {
                                digest_str.push_str(&format!("{:0X}", byte));
                            }
                            use std::ascii::AsciiExt;
                            public_key.trim().eq_ignore_ascii_case(&digest_str)
                        },
                        Err(_) => false
                    },
                    Err(_) => false
                },
                None => false
            }
        });

        match
config.danger_connect_without_providing_domain_for_certificate_verification_and_server_name_indication(stream)
        {
            Ok(ok) => ok,
            Err(_) => {
                println!("Failed to validate certificate");
                return None;
            }
        }
    };

    let mut id = None;
    if let Some(token) = token {
        let packet = Packet::Login(common::Login {
            bot: false,
            name: nick.to_string(),
            password: None,
            token: Some(token.to_string())
        });

        if let Err(err) = common::write(&mut stream, &packet) {
            println!("Could not request login");
            println!("{}", err);
            return None;
        }

        match common::read(&mut stream) {
            Ok(Packet::LoginSuccess(login)) => {
                id = Some(login.id);
                if login.created {
                    println!("Tried to log in with your token: Apparently an account was created.");
                    println!("I think you should stay away from this server. Something is not quite right.");
                    return None;
                }
                println!("Logged in as user #{}", login.id);
            },
            Ok(Packet::Err(code)) => match code {
                common::ERR_LOGIN_INVALID |
                common::ERR_MISSING_FIELD => {},
                common::ERR_LOGIN_BANNED => {
                    println!("You have been banned from this server. :(");
                    return None;
                },
                common::ERR_LOGIN_BOT => {
                    println!("This account is a bot account");
                    return None;
                },
                common::ERR_LIMIT_REACHED => {
                    println!("Username too long");
                    return None;
                },
                common::ERR_MAX_CONN_PER_IP => {
                    println!("Too many connections made from this IP");
                    return None;
                },
                _ => {
                    println!("The server responded with an invalid error.");
                    return None;
                }
            },
            Ok(_) => {
                println!("The server responded with an invalid packet.");
                return None;
            }
            Err(err) => {
                println!("Failed to read from server");
                println!("{}", err);
                return None;
            }
        }
    }

    if id.is_none() {
        println!("If you don't have an account, choose a new password here.");
        println!("Otherwise, enter your existing one.");
        println!("Password: ");
        let pass = readpass!({ return None; });

        let packet = Packet::Login(common::Login {
            bot: false,
            name: nick.to_string(),
            password: Some(pass),
            token: None
        });

        if let Err(err) = common::write(&mut stream, &packet) {
            println!("Could not request login");
            println!("{}", err);
            return None;
        }

        match common::read(&mut stream) {
            Ok(Packet::LoginSuccess(login)) => {
                db.execute(
                    "UPDATE servers SET token = ? WHERE ip = ?",
                    &[&login.token, &addr.to_string()]
                ).unwrap();
                if login.created {
                    println!("Account created");
                }
                id = Some(login.id);
                println!("Logged in as user #{}", login.id);
            },
            Ok(Packet::Err(code)) => match code {
                common::ERR_LOGIN_INVALID => {
                    println!("Invalid credentials");
                    return None;
                },
                common::ERR_LOGIN_BANNED => {
                    println!("You have been banned from this server. :(");
                    return None;
                },
                common::ERR_LOGIN_BOT => {
                    println!("This account is a bot account");
                    return None;
                },
                common::ERR_LIMIT_REACHED => {
                    println!("Username too long");
                    return None;
                },
                _ => {
                    println!("The server responded with an invalid error.");
                    return None;
                }
            },
            Ok(_) => {
                println!("The server responded with an invalid packet.");
                return None;
            }
            Err(err) => {
                println!("Failed to read from server");
                println!("{}", err);
                return None;
            }
        }
    }
    stream.get_ref().set_nonblocking(true).expect("Failed to make stream non-blocking");
    Some(Session::new(addr, id.unwrap(), stream))
}
