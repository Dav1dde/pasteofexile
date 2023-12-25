use crate::{
    consts, net,
    request_context::{Env, FromEnv},
};

impl FromEnv for sentry::Options {
    fn from_env(env: &Env) -> Option<Self> {
        let project = env.var(consts::ENV_SENTRY_PROJECT)?;
        let token = env.var(consts::ENV_SENTRY_TOKEN)?;
        Some(Self { project, token })
    }
}

pub struct Transport(pub worker::Context);

impl sentry::Transport for Transport {
    fn send(&self, url: String, auth: String, content: Vec<u8>) {
        worker::console_warn!("{:?}", std::str::from_utf8(&content));

        self.0.wait_until(async move {
            let response = net::Request::post(url)
                .header("Content-Type", "application/x-sentry-envelope")
                .header("User-Agent", "pobb.bin/1.0")
                .header("X-Sentry-Auth", &auth)
                .body_u8(&content)
                .no_sentry()
                .send()
                .await;

            match response {
                Err(err) => worker::console_log!("failed to send envelope: {:?}", err),
                Ok(mut response) => {
                    if response.status_code() >= 300 {
                        worker::console_log!(
                            "failed to send envelope: {:?}",
                            response.status_code()
                        );
                        if cfg!(feature = "debug") {
                            worker::console_log!(
                                "response: {}",
                                response.text().await.unwrap_or_default()
                            );
                        }
                    }
                }
            }
        });
    }
}
