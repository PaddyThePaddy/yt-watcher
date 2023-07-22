#![allow(non_snake_case)]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// "\s*(pub )?(.+?,)$"
// "    pub $2"

#[derive(Serialize, Deserialize, Debug)]
pub struct Localization {
    pub title: String,
    pub description: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PageInfo {
    pub totalResults: usize,
    pub resultsPerPage: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultipleItemsResponse<T> {
    pub kind: String,
    pub etag: String,
    pub nextPageToken: Option<String>,
    pub prevPageToken: Option<String>,
    pub pageInfo: PageInfo,
    pub items: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

pub mod Channel {
    use super::*;
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContentOwnerDetails {
        pub contentOwner: Option<String>,
        pub timeLinked: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct AuditDetails {
        pub overallGoodStanding: bool,
        pub communityGuidelinesGoodStanding: bool,
        pub copyrightStrikesGoodStanding: bool,
        pub contentIdClaimsGoodStanding: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BrandingSettingsWatch {
        pub textColor: String,
        pub backgroundColor: String,
        pub featuredPlaylistId: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BrandingSettingsChannel {
        pub title: String,
        pub description: String,
        pub keywords: String,
        pub trackingAnalyticsAccountId: Option<String>,
        pub moderateComments: Option<bool>,
        pub unsubscribedTrailer: String,
        pub defaultLanguage: Option<String>,
        pub country: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BrandingSettings {
        pub channel: BrandingSettingsChannel,
        pub watch: Option<BrandingSettingsWatch>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        pub privacyStatus: String,
        pub isLinked: bool,
        pub longUploadsStatus: String,
        pub madeForKids: Option<bool>,
        pub selfDeclaredMadeForKids: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TopicDetails {
        pub topicIds: Vec<String>,
        pub topicCategories: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Statistics {
        pub viewCount: String,
        pub subscriberCount: String,
        pub hiddenSubscriberCount: bool,
        pub videoCount: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RelatedPlaylists {
        pub likes: String,
        pub favorites: Option<String>,
        pub uploads: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContentDetails {
        pub relatedPlaylists: RelatedPlaylists,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Snippet {
        pub title: String,
        pub description: String,
        pub customUrl: String,
        pub publishedAt: String,
        pub thumbnails: HashMap<String, Thumbnail>,
        pub defaultLanguage: Option<String>,
        pub localized: Localization,
        pub country: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resource {
        pub kind: String,
        pub etag: String,
        pub id: String,
        pub snippet: Option<Snippet>,
        pub contentDetails: Option<ContentDetails>,
        pub statistics: Option<Statistics>,
        pub topicDetails: Option<TopicDetails>,
        pub status: Option<Status>,
        pub brandingSettings: Option<BrandingSettings>,
        pub auditDetails: Option<AuditDetails>,
        pub contentOwnerDetails: Option<ContentOwnerDetails>,
        pub localizations: Option<Localization>,
    }
}

pub mod PlayListItem {
    use super::*;
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        pub privacyStatus: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContentDetails {
        pub videoId: String,
        pub startAt: Option<String>,
        pub endAt: Option<String>,
        pub note: Option<String>,
        pub videoPublishedAt: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ResourceId {
        pub kind: String,
        pub videoId: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Snippet {
        pub publishedAt: String,
        pub channelId: String,
        pub title: String,
        pub description: String,
        pub thumbnails: HashMap<String, Thumbnail>,
        pub channelTitle: String,
        pub videoOwnerChannelTitle: String,
        pub videoOwnerChannelId: String,
        pub playlistId: String,
        pub position: usize,
        pub resourceId: ResourceId,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resource {
        pub kind: String,
        pub etag: String,
        pub id: String,
        pub snippet: Option<Snippet>,
        pub contentDetails: Option<ContentDetails>,
        pub status: Option<Status>,
    }
}

pub mod Video {
    use super::*;
    #[derive(Serialize, Deserialize, Debug)]
    pub struct LiveStreamingDetails {
        pub actualStartTime: Option<String>,
        pub actualEndTime: Option<String>,
        pub scheduledStartTime: Option<String>,
        pub scheduledEndTime: Option<String>,
        pub concurrentViewers: Option<String>,
        pub activeLiveChatId: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TagSuggestions {
        pub tag: String,
        pub categoryRestricts: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Suggestions {
        pub processingErrors: Vec<String>,
        pub ProcessingWarnings: Vec<String>,
        pub processingHints: Vec<String>,
        pub tagSuggestions: Vec<TagSuggestions>,
        pub editorSuggestions: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProcessingProgress {
        pub partsTotal: u64,
        pub partsProcessed: u64,
        pub timeLeftMs: u64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProcessingDetails {
        pub processingStatus: String,
        pub processingProgress: ProcessingProgress,
        pub processingFailureReason: String,
        pub fileDetailsAvailability: String,
        pub processingIssuesAvailability: String,
        pub tagSuggestionsAvailability: String,
        pub editorSuggestionsAvailability: String,
        pub thumbnailsAvailability: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct VideoStream {
        pub widthPixels: usize,
        pub heightPixels: usize,
        pub frameRateFps: f64,
        pub aspectRatio: f64,
        pub codec: String,
        pub bitrateBps: u64,
        pub rotation: String,
        pub vendor: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct AudioStream {
        pub channelCount: usize,
        pub codec: String,
        pub bitrateBps: u64,
        pub vendor: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FileDetails {
        pub fileName: String,
        pub fileSize: u64,
        pub fileType: String,
        pub container: String,
        pub videoStreams: Vec<VideoStream>,
        pub audioStreams: Vec<AudioStream>,
        pub durationMs: u64,
        pub bitrateBps: u64,
        pub creationTime: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RecordingDetails {
        pub recordingDate: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TopicDetails {
        pub topicIds: Vec<String>,
        pub relevantTopicIds: Vec<String>,
        pub topicCategories: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Player {
        pub embedHtml: String,
        pub embedHeight: Option<u64>,
        pub embedWidth: Option<u64>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Statistics {
        pub viewCount: String,
        pub likeCount: String,
        pub dislikeCount: Option<String>,
        pub favoriteCount: String,
        pub commentCount: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        pub uploadStatus: String,
        pub failureReason: Option<String>,
        pub rejectionReason: Option<String>,
        pub privacyStatus: String,
        pub publishAt: Option<String>,
        pub license: String,
        pub embeddable: bool,
        pub publicStatsViewable: bool,
        pub madeForKids: bool,
        pub selfDeclaredMadeForKids: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContentRating {
        pub acbRating: Option<String>,
        pub agcomRating: Option<String>,
        pub anatelRating: Option<String>,
        pub bbfcRating: Option<String>,
        pub bfvcRating: Option<String>,
        pub bmukkRating: Option<String>,
        pub catvRating: Option<String>,
        pub catvfrRating: Option<String>,
        pub cbfcRating: Option<String>,
        pub cccRating: Option<String>,
        pub cceRating: Option<String>,
        pub chfilmRating: Option<String>,
        pub chvrsRating: Option<String>,
        pub cicfRating: Option<String>,
        pub cnaRating: Option<String>,
        pub cncRating: Option<String>,
        pub csaRating: Option<String>,
        pub cscfRating: Option<String>,
        pub czfilmRating: Option<String>,
        pub djctqRating: Option<String>,
        pub ecbmctRating: Option<String>,
        pub eefilmRating: Option<String>,
        pub egfilmRating: Option<String>,
        pub eirinRating: Option<String>,
        pub fcbmRating: Option<String>,
        pub fcoRating: Option<String>,
        pub fmocRating: Option<String>,
        pub fpbRating: Option<String>,
        pub fskRating: Option<String>,
        pub grfilmRating: Option<String>,
        pub icaaRating: Option<String>,
        pub ifcoRating: Option<String>,
        pub ilfilmRating: Option<String>,
        pub incaaRating: Option<String>,
        pub kfcbRating: Option<String>,
        pub kijkwijzerRating: Option<String>,
        pub kmrbRating: Option<String>,
        pub lsfRating: Option<String>,
        pub mccaaRating: Option<String>,
        pub mccypRating: Option<String>,
        pub mcstRating: Option<String>,
        pub mdaRating: Option<String>,
        pub medietilsynetRating: Option<String>,
        pub mekuRating: Option<String>,
        pub mibacRating: Option<String>,
        pub mocRating: Option<String>,
        pub moctwRating: Option<String>,
        pub mpaaRating: Option<String>,
        pub mpaatRating: Option<String>,
        pub mtrcbRating: Option<String>,
        pub nbcRating: Option<String>,
        pub nbcplRating: Option<String>,
        pub nfrcRating: Option<String>,
        pub nfvcbRating: Option<String>,
        pub nkclvRating: Option<String>,
        pub oflcRating: Option<String>,
        pub pefilmRating: Option<String>,
        pub rcnofRating: Option<String>,
        pub resorteviolenciaRating: Option<String>,
        pub rtcRating: Option<String>,
        pub rteRating: Option<String>,
        pub russiaRating: Option<String>,
        pub skfilmRating: Option<String>,
        pub smaisRating: Option<String>,
        pub smsaRating: Option<String>,
        pub tvpgRating: Option<String>,
        pub ytRating: Option<String>,
        pub djctqRatingReasons: Option<Vec<String>>,
        pub fpbRatingReasons: Option<Vec<String>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RegionRestriction {
        pub allowed: Vec<String>,
        pub blocked: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContentDetails {
        pub duration: String,
        pub dimension: String,
        pub definition: String,
        pub caption: String,
        pub licensedContent: bool,
        pub regionRestriction: Option<RegionRestriction>,
        pub contentRation: ContentRating,
        pub projection: String,
        pub hasCustomThumbnail: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Snippet {
        pub publishedAt: String,
        pub channelId: String,
        pub title: String,
        pub description: String,
        pub thumbnails: HashMap<String, Thumbnail>,
        pub channelTitle: String,
        pub tags: Option<Vec<String>>,
        pub categoryId: String,
        pub liveBroadcastContent: String,
        pub defaultLanguage: Option<String>,
        pub localized: Localization,
        pub defaultAudioLanguage: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resource {
        pub kind: String,
        pub etag: String,
        pub id: String,
        pub snippet: Option<Snippet>,
        pub contentDetails: Option<ContentDetails>,
        pub status: Option<Status>,
        pub statistics: Option<Statistics>,
        pub player: Option<Player>,
        pub topicDetails: Option<TopicDetails>,
        pub recordingDetails: Option<RecordingDetails>,
        pub fileDetails: Option<FileDetails>,
        pub processingDetails: Option<ProcessingDetails>,
        pub suggestions: Option<Suggestions>,
        pub liveStreamingDetails: Option<LiveStreamingDetails>,
        pub localizations: Option<HashMap<String, Localization>>,
    }
}
