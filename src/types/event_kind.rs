use serde::de::Error as DeError;
use serde::de::{Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Context, Readable, Reader, Writable, Writer};
use std::convert::From;
use std::fmt;

macro_rules! define_event_kinds {
    ($($comment:expr, $name:ident = $value:expr),*) => {
        /// A kind of Event
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[repr(u32)]
        pub enum EventKind {
            $(
                #[doc = $comment]
                $name = $value,
            )*
            /// Job Request (NIP-90) 5000-5999
            JobRequest(u32),
            /// Job Result (NIP-90) 6000-6999
            JobResult(u32),
            /// Relay-specific replaceable event
            Replaceable(u32),
            /// Ephemeral event, sent to all clients with matching filters and should not be stored
            Ephemeral(u32),
            /// Something else?
            Other(u32),
        }

        static WELL_KNOWN_KINDS: &[EventKind] = &[
            $($name,)*
        ];

        impl From<u32> for EventKind {
            fn from(u: u32) -> Self {
                match u {
                    $($value => $name,)*
                    x if (5_000..5_999).contains(&x) => JobRequest(x),
                    x if (6_000..6_999).contains(&x) => JobResult(x),
                    x if (10_000..20_000).contains(&x) => Replaceable(x),
                    x if (20_000..30_000).contains(&x) => Ephemeral(x),
                    x => Other(x),
                }
            }
        }

        impl From<EventKind> for u32 {
            fn from(e: EventKind) -> u32 {
                match e {
                    $($name => $value,)*
                    JobRequest(u) => u,
                    JobResult(u) => u,
                    Replaceable(u) => u,
                    Ephemeral(u) => u,
                    Other(u) => u,
                }
            }
        }
    };
}

define_event_kinds!(
    "Event sets the metadata associated with a public key (NIP-01)",
    Metadata = 0,
    "Event is a text note (NIP-01)",
    TextNote = 1,
    "Event contains a relay URL which the author recommends",
    RecommendRelay = 2,
    "Event contains tags which represent the authors contacts including the authors pet names for them (NIP-02)",
    ContactList = 3,
    "Event is an encrypted direct message (NIP-04)",
    EncryptedDirectMessage = 4,
    "Event is an authors request to delete previous events (NIP-09)",
    EventDeletion = 5,
    "Repost (NIP-18)",
    Repost = 6,
    "Event is a reaction to a `TextNote` event (NIP-25)",
    Reaction = 7,
    "Badge Award (NIP-58)",
    BadgeAward = 8,
    "Seal (NIP-59 PR 716)",
    Seal = 13,
    "Chat Message / DM (NIP-24 PR 686)",
    DmChat = 14,
    "Generic Repost (NIP-18)",
    GenericRepost = 16,
    "Event creates a public channel (NIP-28)",
    ChannelCreation = 40,
    "Event sets metadata on a public channel (NIP-28)",
    ChannelMetadata = 41,
    "Event creates a message on a public channel (NIP-28)",
    ChannelMessage = 42,
    "Event hides a message on a public channel (NIP-28)",
    ChannelHideMessage = 43,
    "Event mutes a user on a public channel (NIP-28)",
    ChannelMuteUser = 44,
    "Reserved for future public channel usage",
    PublicChatReserved45 = 45,
    "Reserved for future public channel usage",
    PublicChatReserved46 = 46,
    "Reserved for future public channel usage",
    PublicChatReserved47 = 47,
    "Reserved for future public channel usage",
    PublicChatReserved48 = 48,
    "Reserved for future public channel usage",
    PublicChatReserved49 = 49,
    "Timestamps",
    Timestamp = 1040,
    "Gift Wrap (NIP-59 PR 716)",
    GiftWrap = 1059,
    "File Metadata (NIP-94)",
    FileMetadata = 1063,
    "Live Chat Message (NIP-53)",
    LiveChatMessage = 1311,
    "Problem Tracker (nostrocket-1971)",
    ProblemTracker = 1971,
    "Reporting (NIP-56)",
    Reporting = 1984,
    "Label (NIP-32)",
    Label = 1985,
    "Community (exclusive) Post (NIP-72 pr 753)",
    CommunityPost = 4549,
    "Community Post Approval (NIP-72)",
    CommunityPostApproval = 4550,
    "Job Feedback (NIP-90)",
    JobFeedback = 7000,
    "Zap Goal (NIP-75)",
    ZapGoal = 9041,
    "Zap Request",
    ZapRequest = 9734,
    "Zap",
    Zap = 9735,
    "Highlights (NIP-84)",
    Highlights = 9802,
    "Mute List (NIP-51)",
    MuteList = 10000,
    "PinList (NIP-51)",
    PinList = 10001,
    "Relays List (NIP-65)",
    RelayList = 10002,
    "Bookmarks List (NIP-51)",
    BookmarkList = 10003,
    "Communities List (NIP-51)",
    CommunityList = 10004,
    "Public Chats List (NIP-51)",
    PublicChatsList = 10005,
    "Blocked Relays List (NIP-51)",
    BlockedRelaysList = 10006,
    "Search Relays List (NIP-51)",
    SearchRelaysList = 10007,
    "Interests List (NIP-51)",
    InterestsList = 10015,
    "USer Emoji List (NIP-51)",
    UserEmojiList = 10030,
    "Wallet Info (NIP-47)",
    WalletInfo = 13194,
    "Client Authentication (NIP-42)",
    Auth = 22242,
    "Wallet Request (NIP-47)",
    WalletRequest = 23194,
    "Wallet Response (NIP-47)",
    WalletResponse = 23195,
    "Nostr Connect (NIP-46)",
    NostrConnect = 24133,
    "HTTP Auth (NIP-98)",
    HttpAuth = 27235,
    "Categorized People List (NIP-51)",
    FollowSets = 30000,
    "Categorized Bookmark List (NIP-51)",
    GenericSets = 30001,
    "Relay Sets (NIP-51)",
    RelaySets = 30002,
    "Bookmark Sets (NIP-51)",
    BookmarkSets = 30003,
    "Curation Sets (NIP-51)",
    CurationSets = 30004,
    "Profile Badges (NIP-58)",
    ProfileBadges = 30008,
    "Badge Definition (NIP-58)",
    BadgeDefinition = 30009,
    "Interest Sets (NIP-51)",
    InterestSets = 30015,
    "Create or update a stall (NIP-15)",
    CreateUpdateStall = 30017,
    "Create or update a product (NIP-15)",
    CreateUpdateProduct = 30018,
    "Long-form Content (NIP-23)",
    LongFormContent = 30023,
    "Draft Long-form Content (NIP-23)",
    DraftLongFormContent = 30024,
    "Emoji Sets (NIP-51)",
    EmojiSets = 30030,
    "Application Specific Data, (NIP-78)",
    AppSpecificData = 30078,
    "Live Event (NIP-53)",
    LiveEvent = 30311,
    "User Status (NIP-315 PR 737)",
    UserStatus = 30315,
    "Classified Listing (NIP-99)",
    ClassifiedListing = 30402,
    "Draft Classified Listing (NIP-99)",
    DraftClassifiedListing = 30403,
    "Date-Based Calendar Event (NIP-52)",
    DateBasedCalendarEvent = 31922,
    "Time-Based Calendar Event (NIP-52)",
    TimeBasedCalendarEvent = 31923,
    "Calendar (NIP-52)",
    Calendar = 31924,
    "Calendar Event RSVP (NIP-52)",
    CalendarEventRsvp = 31925,
    "Handler Recommendation (NIP-89)",
    HandlerRecommendation = 31989,
    "Handler Information (NIP-89)",
    HandlerInformation = 31990,
    "Community Definition (NIP-72)",
    CommunityDefinition = 34550
);

use EventKind::*;

impl EventKind {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventKind {
        TextNote
    }

    /// Is a job request kind
    pub fn is_job_request(&self) -> bool {
        let u: u32 = From::from(*self);
        (5000..=5999).contains(&u)
    }

    /// Is a job result kind
    pub fn is_job_result(&self) -> bool {
        let u: u32 = From::from(*self);
        (6000..=6999).contains(&u)
    }

    /// If this event kind is a replaceable event
    /// NOTE: this INCLUDES parameterized replaceable events
    pub fn is_replaceable(&self) -> bool {
        match *self {
            Metadata => true,
            ContactList => true,
            _ => {
                let u: u32 = From::from(*self);
                (10000..=19999).contains(&u) || (30000..=39999).contains(&u)
            }
        }
    }

    /// If this event kind is ephemeral
    pub fn is_ephemeral(&self) -> bool {
        let u: u32 = From::from(*self);
        (20000..=29999).contains(&u)
    }

    /// If this event kind is parameterized replaceable
    pub fn is_parameterized_replaceable(&self) -> bool {
        let u: u32 = From::from(*self);
        (30000..=39999).contains(&u)
    }

    /// If this event kind is feed related.
    pub fn is_feed_related(&self) -> bool {
        self.is_feed_displayable() || self.augments_feed_related()
    }

    /// If this event kind is feed displayable.
    pub fn is_feed_displayable(&self) -> bool {
        matches!(
            *self,
            TextNote
                | EncryptedDirectMessage
                | Repost
                | DmChat
                | GenericRepost
                | ChannelMessage
                | FileMetadata
                | LiveChatMessage
                | CommunityPost
                | LongFormContent
                | DraftLongFormContent
        )
    }

    /// Is direct message related
    pub fn is_direct_message_related(&self) -> bool {
        matches!(*self, EncryptedDirectMessage | DmChat | GiftWrap)
    }

    /// If this event kind augments a feed related event
    pub fn augments_feed_related(&self) -> bool {
        matches!(
            *self,
            EventDeletion | Reaction | Timestamp | Label | Reporting | Zap
        )
    }

    /// If the contents are expected to be encrypted (or empty)
    pub fn contents_are_encrypted(&self) -> bool {
        matches!(
            *self,
            EncryptedDirectMessage
                | MuteList
                | PinList
                | BookmarkList
                | CommunityList
                | PublicChatsList
                | BlockedRelaysList
                | SearchRelaysList
                | InterestsList
                | UserEmojiList
                | JobRequest(_)
                | JobResult(_)
                | WalletRequest
                | WalletResponse
                | NostrConnect
        )
    }

    /// This iterates through every well-known EventKind
    pub fn iter() -> EventKindIterator {
        EventKindIterator::new()
    }
}

/// Iterator over well known `EventKind`s
#[derive(Clone, Copy, Debug)]
pub struct EventKindIterator {
    pos: usize,
}

impl EventKindIterator {
    fn new() -> EventKindIterator {
        EventKindIterator { pos: 0 }
    }
}

impl Iterator for EventKindIterator {
    type Item = EventKind;

    fn next(&mut self) -> Option<EventKind> {
        if self.pos == WELL_KNOWN_KINDS.len() {
            None
        } else {
            let rval = WELL_KNOWN_KINDS[self.pos];
            self.pos += 1;
            Some(rval)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.pos, Some(WELL_KNOWN_KINDS.len()))
    }
}

impl Serialize for EventKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let u: u32 = From::from(*self);
        serializer.serialize_u32(u)
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(EventKindVisitor)
    }
}

struct EventKindVisitor;

impl Visitor<'_> for EventKindVisitor {
    type Value = EventKind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an unsigned number that matches a known EventKind")
    }

    fn visit_u32<E>(self, v: u32) -> Result<EventKind, E>
    where
        E: DeError,
    {
        Ok(From::<u32>::from(v))
    }

    // JsonValue numbers come in as u64
    fn visit_u64<E>(self, v: u64) -> Result<EventKind, E>
    where
        E: DeError,
    {
        Ok(From::<u32>::from(v as u32))
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: Context> Readable<'a, C> for EventKind {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let value = u32::read_from(reader)?;
        Ok(value.into())
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <u32 as Readable<'a, C>>::minimum_bytes_needed()
    }
}

#[cfg(feature = "speedy")]
impl<C: Context> Writable<C> for EventKind {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(u32::from(*self))
    }

    #[inline]
    fn bytes_needed(&self) -> Result<usize, C::Error> {
        Ok(std::mem::size_of::<u32>())
    }
}

/// Either an EventKind or a range (a vector of length 2 with start and end)
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[serde(untagged)]
pub enum EventKindOrRange {
    /// A single EventKind
    EventKind(EventKind),

    /// A range of EventKinds
    // NOTE: the internal Vec should have exactly 2 fields.  To force this with a tuple
    //       struct makes ser/de a bitch, so we don't.
    Range(Vec<EventKind>),
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {EventKind, test_event_kind_serde}

    #[test]
    fn test_replaceable_ephemeral() {
        assert!(Metadata.is_replaceable());
        assert!(!TextNote.is_replaceable());
        assert!(!Zap.is_replaceable());
        assert!(LongFormContent.is_replaceable());

        assert!(!TextNote.is_ephemeral());
        assert!(Auth.is_ephemeral());

        assert!(!TextNote.is_parameterized_replaceable());
        assert!(LongFormContent.is_parameterized_replaceable());
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_event_kind() {
        let ek = EventKind::mock();
        let bytes = ek.write_to_vec().unwrap();
        let ek2 = EventKind::read_from_buffer(&bytes).unwrap();
        assert_eq!(ek, ek2);
    }
}
