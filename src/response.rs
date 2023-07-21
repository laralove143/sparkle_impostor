//! Wrapper types for [`Response`]

use std::mem;

use serde::de::DeserializeOwned;
use twilight_http::Response;

use crate::error::Error;

/// Wrapper type for [`Response`] that can hold deserialized values too
#[derive(Debug)]
pub enum MaybeDeserialized<T> {
    /// Response hasn't been deserialized yet
    Response(Response<T>),
    /// Response is being deserialized
    ///
    /// This is a marker type that should be unreachable
    Deserializing,
    /// Response has been deserialized
    Deserialized(T),
}

impl<T: DeserializeOwned + Unpin + Send> MaybeDeserialized<T> {
    /// Return the deserialized model
    ///
    /// Deserializes the model if it hasn't been deserialized yet, this method
    /// is otherwise actually sync
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeserializeBody`] if deserializing the type fails
    pub async fn model(&mut self) -> Result<&T, Error> {
        match self {
            Self::Response(_) => {
                let self_owned = mem::replace(self, Self::Deserializing);

                let Self::Response(response) = self_owned else {
                    unreachable!();
                };

                let model = response.model().await?;
                *self = Self::Deserialized(model);

                let Self::Deserialized(model_ref) = self else {
                    unreachable!();
                };

                Ok(model_ref)
            }
            Self::Deserializing => unreachable!(),
            Self::Deserialized(model) => Ok(model),
        }
    }
}
