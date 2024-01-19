use super::{Event, EventKind, IdHex, PublicKeyHex, Unixtime};
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};
use std::collections::BTreeMap;
use std::fmt;

/// Filter which specify what events a client is looking for
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Filter {
    /// Events which match these ids
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub ids: Vec<IdHex>, // ID as hex

    /// Events which match these authors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub authors: Vec<PublicKeyHex>, // PublicKey as hex

    /// Events which match these kinds
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKind>,

    /// Events which match the given tags
    #[serde(
        flatten,
        serialize_with = "serialize_tags",
        deserialize_with = "deserialize_tags"
    )]
    pub tags: BTreeMap<char, Vec<String>>,

    /// Events occuring after this date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub since: Option<Unixtime>,

    /// Events occuring before this date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub until: Option<Unixtime>,

    /// A limit on the number of events to return in the initial query
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Filter {
    /// Create a new Filter object
    pub fn new() -> Filter {
        Default::default()
    }

    /// Add an Id to the filter.
    pub fn add_id(&mut self, id_hex: &IdHex) {
        if !self.ids.contains(id_hex) {
            self.ids.push(id_hex.to_owned());
        }
    }

    /// Delete an Id from the filter
    pub fn del_id(&mut self, id_hex: &IdHex) {
        if let Some(index) = self.ids.iter().position(|id| *id == *id_hex) {
            let _ = self.ids.swap_remove(index);
        }
    }

    /// Add a PublicKey to the filter
    pub fn add_author(&mut self, public_key_hex: &PublicKeyHex) {
        if !self.authors.contains(public_key_hex) {
            self.authors.push(public_key_hex.to_owned());
        }
    }

    /// Delete a PublicKey from the filter
    pub fn del_author(&mut self, public_key_hex: &PublicKeyHex) {
        if let Some(index) = self.authors.iter().position(|pk| *pk == *public_key_hex) {
            let _ = self.authors.swap_remove(index);
        }
    }

    /// Add an EventKind to the filter
    pub fn add_event_kind(&mut self, event_kind: EventKind) {
        if self.kinds.contains(&event_kind) {
            return;
        }
        self.kinds.push(event_kind);
    }

    /// Delete an EventKind from the filter
    pub fn del_event_kind(&mut self, event_kind: EventKind) {
        if let Some(position) = self.kinds.iter().position(|&x| x == event_kind) {
            let _ = self.kinds.swap_remove(position);
        }
    }

    /// Add a Tag value to a filter
    pub fn add_tag_value(&mut self, letter: char, value: String) {
        let _ = self
            .tags
            .entry(letter)
            .and_modify(|values| values.push(value.clone()))
            .or_insert(vec![value]);
    }

    /// Add a Tag value from a filter
    pub fn del_tag_value(&mut self, letter: char, value: String) {
        let mut became_empty: bool = false;
        let _ = self.tags.entry(letter).and_modify(|values| {
            if let Some(position) = values.iter().position(|x| *x == value) {
                let _ = values.swap_remove(position);
            }
            if values.is_empty() {
                became_empty = true;
            }
        });
        if became_empty {
            let _ = self.tags.remove(&letter);
        }
    }

    /// Set all values for a given tag
    pub fn set_tag_values(&mut self, letter: char, values: Vec<String>) {
        let _ = self.tags.insert(letter, values);
    }

    /// Remove all Tag values of a given kind from a filter
    pub fn clear_tag_values(&mut self, letter: char) {
        let _ = self.tags.remove(&letter);
    }

    /// This is an INCOMPLETE matching of an event against the filter.
    ///
    /// It is only incomplete because I plan to rewrite how tags work and it makes
    /// sense to do that first.
    pub fn event_matches_incomplete(&self, e: &Event) -> bool {
        if !self.ids.is_empty() {
            let idhex: IdHex = e.id.into();
            if !self.ids.contains(&idhex) {
                return false;
            }
        }

        if !self.authors.is_empty() {
            let pubkeyhex: PublicKeyHex = e.pubkey.into();
            if !self.authors.contains(&pubkeyhex) {
                return false;
            }
        }

        if !self.kinds.is_empty() {
            if !self.kinds.contains(&e.kind) {
                return false;
            }
        }

        // TBD - check tags

        if let Some(since) = self.since {
            if e.created_at < since {
                return false;
            }
        }

        if let Some(until) = self.until {
            if e.created_at > until {
                return false;
            }
        }

        true
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Filter {
        let mut map = BTreeMap::new();
        let _ = map.insert('e', vec![IdHex::mock().to_string()]);
        let _ = map.insert(
            'p',
            vec!["221115830ced1ca94352002485fcc7a75dcfe30d1b07f5f6fbe9c0407cfa59a1".to_string()],
        );

        Filter {
            ids: vec![IdHex::try_from_str(
                "3ab7b776cb547707a7497f209be799710ce7eb0801e13fd3c4e7b9261ac29084",
            )
            .unwrap()],
            authors: vec![],
            kinds: vec![EventKind::TextNote, EventKind::Metadata],
            tags: map,
            since: Some(Unixtime(1668572286)),
            ..Default::default()
        }
    }
}

fn serialize_tags<S>(tags: &BTreeMap<char, Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(tags.len()))?;
    for (tag, values) in tags.iter() {
        map.serialize_entry(&format!("#{tag}"), values)?;
    }
    map.end()
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<BTreeMap<char, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TagsVisitor;

    impl<'de> Visitor<'de> for TagsVisitor {
        type Value = BTreeMap<char, Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map with keys in \"#t\" format")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut tags: BTreeMap<char, Vec<String>> = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, Vec<String>>()? {
                let mut chars = key.chars();
                if let (Some('#'), Some(ch), None) = (chars.next(), chars.next(), chars.next()) {
                    let _ = tags.insert(ch, value);
                }
            }
            Ok(tags)
        }
    }

    deserializer.deserialize_map(TagsVisitor)
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {Filter, test_filters_serde}

    #[test]
    fn test_filter_mock() {
        assert_eq!(
            &serde_json::to_string(&Filter::mock()).unwrap(),
            r##"{"ids":["3ab7b776cb547707a7497f209be799710ce7eb0801e13fd3c4e7b9261ac29084"],"kinds":[1,0],"#e":["5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6"],"#p":["221115830ced1ca94352002485fcc7a75dcfe30d1b07f5f6fbe9c0407cfa59a1"],"since":1668572286}"##
        );
    }

    #[test]
    fn test_add_remove_id() {
        let mock = IdHex::mock();

        let mut filters: Filter = Filter::new();

        filters.add_id(&mock);
        assert_eq!(filters.ids.len(), 1);
        filters.add_id(&mock); // overwrites
        assert_eq!(filters.ids.len(), 1);
        filters.del_id(&mock);
        assert!(filters.ids.is_empty());
    }

    // add_remove_author would be very similar to the above

    #[test]
    fn test_add_remove_tags() {
        let mut filter = Filter::mock();
        filter.del_tag_value('e', IdHex::mock().to_string());
        assert_eq!(filter.tags.get(&'e'), None);

        filter.add_tag_value('t', "footstr".to_string());
        filter.add_tag_value('t', "bitcoin".to_string());
        filter.del_tag_value('t', "bitcoin".to_string());
        assert!(filter.tags.get(&'t').is_some());
    }

    #[test]
    fn test_event_matches() {
        use crate::{Id, KeySigner, PreEvent, PrivateKey, Signer, Tag, UncheckedUrl};

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let preevent = PreEvent {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::Event {
                    id: Id::mock(),
                    recommended_relay_url: Some(UncheckedUrl::mock()),
                    marker: None,
                    trailing: Vec::new(),
                },
                Tag::Hashtag {
                    hashtag: "foodstr".to_string(),
                    trailing: Vec::new(),
                },
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event(preevent).unwrap();

        let mut filter = Filter {
            authors: vec![signer.public_key().into()],
            ..Default::default()
        };
        filter.add_tag_value('e', Id::mock().as_hex_string());
        assert_eq!(filter.event_matches_incomplete(&event), true);

        let filter = Filter {
            authors: vec![signer.public_key().into()],
            kinds: vec![EventKind::LongFormContent],
            ..Default::default()
        };
        assert_eq!(filter.event_matches_incomplete(&event), false);

        let filter = Filter {
            ids: vec![IdHex::mock()],
            authors: vec![signer.public_key().into()],
            ..Default::default()
        };
        assert_eq!(filter.event_matches_incomplete(&event), false);
    }
}
