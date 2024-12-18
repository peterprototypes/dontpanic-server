use std::future::{ready, Ready};

use actix_session::Session;
use actix_session::SessionExt;
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use sea_orm::prelude::*;

use crate::entity::prelude::Users;
use crate::entity::users;

use crate::AppContext;
use crate::Error;

#[derive(Clone)]
pub struct Identity {
    pub user_id: u32,
    session: Session,
}

impl Identity {
    pub async fn user(&self, ctx: &AppContext<'_>) -> Result<users::Model, Error> {
        let user = Users::find_by_id(self.user_id)
            .one(&ctx.db)
            .await?
            .ok_or(Error::LoginRequired)?;

        Ok(user)
    }

    pub fn logout(&self) {
        self.session.remove("uid");
    }
}

impl FromRequest for Identity {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let session = req.get_session();

        let Ok(Some(user_id)) = session.get::<u32>("uid") else {
            return ready(Err(Error::LoginRequired));
        };

        ready(Ok(Identity { user_id, session }))
    }
}
