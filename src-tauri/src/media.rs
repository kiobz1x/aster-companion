use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct NowPlayingInfo {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub app: String,
}

#[cfg(windows)]
async fn read_smtc() -> Result<NowPlayingInfo, String> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())?;

    let session = match manager.GetCurrentSession() {
        Ok(s) => s,
        Err(_) => return Ok(NowPlayingInfo::default()),
    };

    let playback = session
        .GetPlaybackInfo()
        .and_then(|p| p.PlaybackStatus())
        .map(|st| st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false);

    let app = session
        .SourceAppUserModelId()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let props = match session.TryGetMediaPropertiesAsync() {
        Ok(op) => op.await.ok(),
        Err(_) => None,
    };

    let (title, artist) = if let Some(p) = props {
        (
            p.Title().map(|s| s.to_string()).unwrap_or_default(),
            p.Artist().map(|s| s.to_string()).unwrap_or_default(),
        )
    } else {
        (String::new(), String::new())
    };

    Ok(NowPlayingInfo {
        playing: playback && (!title.trim().is_empty() || !artist.trim().is_empty()),
        title: title.trim().to_string(),
        artist: artist.trim().to_string(),
        app,
    })
}

#[tauri::command]
pub async fn get_now_playing() -> NowPlayingInfo {
    #[cfg(windows)]
    {
        read_smtc().await.unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        NowPlayingInfo::default()
    }
}
