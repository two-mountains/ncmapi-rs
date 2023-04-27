use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Display};

pub struct ApiResponse {
    data: Vec<u8>,
}

impl From<ApiResponse> for Vec<u8> {
    fn from(value: ApiResponse) -> Self {
        value.data
    }
}

impl Display for ApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.data()))
    }
}

impl ApiResponse {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn deserialize_to_implict(&self) -> ImplicitResult {
        serde_json::from_slice::<ImplicitResult>(self.data()).unwrap()
    }

    pub fn deserialize<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: Serialize + Deserialize<'a>,
    {
        serde_json::from_slice::<T>(self.data())
    }
}

impl fmt::Debug for ApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiResponse")
            .field("data", &self.to_string())
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImplicitResult {
    #[serde(default)]
    pub code: usize,

    #[serde(default)]
    pub msg: Value,

    #[serde(default)]
    pub message: Value,

    #[serde(default)]
    pub time: usize,

    #[serde(default)]
    pub result: Value,

    #[serde(default)]
    pub data: Value,
}
