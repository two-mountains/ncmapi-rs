use serde_json::json;

use crate::{
    AccountBody, ApiResponse, NcmClient, Page, Result, UserDetailsBody, UserId, UserPlaylistsBody,
    transport::{ACCOUNT, RequestPlanBuilder, USER_PLAYLISTS, user_details},
};

/// User-account and profile endpoints exposed by [`NcmClient::profile`].
pub struct Profile<'client> {
    client: &'client NcmClient,
}

impl NcmClient {
    pub fn profile(&self) -> Profile<'_> {
        Profile { client: self }
    }
}

impl<'client> Profile<'client> {
    pub async fn current(&self) -> Result<ApiResponse<AccountBody>> {
        self.client
            .execute(RequestPlanBuilder::post(ACCOUNT).build())
            .await?
            .decode()
    }

    pub async fn details(&self, user: UserId) -> Result<ApiResponse<UserDetailsBody>> {
        let url = user_details(user.0);
        self.client
            .execute(RequestPlanBuilder::post(&url).build())
            .await?
            .decode()
    }

    pub async fn playlists(
        &self,
        user: UserId,
        page: Page,
    ) -> Result<ApiResponse<UserPlaylistsBody>> {
        let request = RequestPlanBuilder::post(USER_PLAYLISTS)
            .payload(json!({
                "uid": user.0,
                "includeVideo": true,
                "limit": page.limit(),
                "offset": page.offset(),
            }))
            .build();

        self.client.execute(request).await?.decode()
    }
}
