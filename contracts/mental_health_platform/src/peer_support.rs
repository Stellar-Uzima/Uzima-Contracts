use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{contracttype, Address, Env, String, Vec};

pub struct PeerSupportManager;

impl PeerSupportManager {
    pub fn create_peer_group(
        env: &Env,
        group_id: String,
        name: String,
        description: String,
        focus_area: String,
        moderator: Address,
        max_members: u32,
        privacy_level: GroupPrivacy,
    ) -> Result<(), Error> {
        let group = PeerGroup {
            group_id: group_id.clone(),
            name,
            description,
            focus_area,
            moderator,
            members: Vec::new(env),
            max_members,
            privacy_level,
            rules: Vec::new(env),
            created_timestamp: env.ledger().timestamp(),
        };

        env.storage().instance().set(&group_id, &group);

        env.events().publish(
            (Symbol::new(env, "peer_group_created"),),
            (group_id, moderator),
        );

        Ok(())
    }

    pub fn join_peer_group(env: &Env, group_id: String, user: Address) -> Result<(), Error> {
        let mut group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        // Check if group is full
        if group.members.len() >= group.max_members {
            return Err(Error::InvalidInput); // Group full
        }

        // Check if user is already a member
        for member in group.members.iter() {
            if member == user {
                return Err(Error::InvalidInput); // Already member
            }
        }

        group.members.push_back(user.clone());
        env.storage().instance().set(&group_id, &group);

        env.events()
            .publish((Symbol::new(env, "user_joined_group"),), (group_id, user));

        Ok(())
    }

    pub fn leave_peer_group(env: &Env, group_id: String, user: Address) -> Result<(), Error> {
        let mut group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        let mut new_members = Vec::new(env);
        let mut was_member = false;

        for member in group.members.iter() {
            if member != user {
                new_members.push_back(member);
            } else {
                was_member = true;
            }
        }

        if !was_member {
            return Err(Error::InvalidInput); // Not a member
        }

        group.members = new_members;
        env.storage().instance().set(&group_id, &group);

        env.events()
            .publish((Symbol::new(env, "user_left_group"),), (group_id, user));

        Ok(())
    }

    pub fn post_message(
        env: &Env,
        group_id: String,
        sender: Address,
        content: String,
        message_type: MessageType,
    ) -> Result<u64, Error> {
        // Verify sender is member of group
        let group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        let mut is_member = false;
        for member in group.members.iter() {
            if member == sender {
                is_member = true;
                break;
            }
        }

        if !is_member && sender != group.moderator {
            return Err(Error::Unauthorized);
        }

        let message_id = env.ledger().timestamp() as u64;

        let message = PeerMessage {
            message_id,
            group_id: group_id.clone(),
            sender,
            timestamp: env.ledger().timestamp(),
            content,
            message_type,
            moderated: false,
        };

        // Store message (in a real implementation, this would be more sophisticated)
        let messages_key = String::from_str(env, "group_messages");
        let mut messages: Vec<PeerMessage> = env
            .storage()
            .instance()
            .get(&messages_key)
            .unwrap_or(Vec::new(env));
        messages.push_back(message);
        env.storage().instance().set(&messages_key, &messages);

        // Check for crisis keywords in message
        if Self::contains_crisis_keywords(env, message_type, content.clone()) {
            Self::flag_crisis_message(env, message_id, group_id.clone(), sender);
        }

        env.events().publish(
            (Symbol::new(env, "peer_message_posted"),),
            PeerMessageEvent {
                message_id,
                group_id,
                sender,
                message_type: String::from_str(env, "message"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(message_id)
    }

    pub fn get_group_messages(
        env: &Env,
        group_id: String,
        user: Address,
        limit: Option<u32>,
    ) -> Result<Vec<PeerMessage>, Error> {
        // Verify user has access to group
        let group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        let mut has_access = false;
        for member in group.members.iter() {
            if member == user {
                has_access = true;
                break;
            }
        }

        if !has_access && user != group.moderator {
            return Err(Error::Unauthorized);
        }

        let messages_key = String::from_str(env, "group_messages");
        let mut messages: Vec<PeerMessage> = env
            .storage()
            .instance()
            .get(&messages_key)
            .unwrap_or(Vec::new(env));

        // Return most recent messages
        if let Some(limit) = limit {
            if messages.len() > limit {
                let start = messages.len() - limit;
                let mut result = Vec::new(env);
                for i in start..messages.len() {
                    result.push_back(messages.get(i).unwrap());
                }
                messages = result;
            }
        }

        Ok(messages)
    }

    pub fn moderate_message(
        env: &Env,
        group_id: String,
        message_id: u64,
        moderator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        let group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        if moderator != group.moderator {
            return Err(Error::Unauthorized);
        }

        let messages_key = String::from_str(env, "group_messages");
        let mut messages: Vec<PeerMessage> = env
            .storage()
            .instance()
            .get(&messages_key)
            .unwrap_or(Vec::new(env));

        for i in 0..messages.len() {
            let mut message = messages.get(i).unwrap();
            if message.message_id == message_id {
                message.moderated = true;
                messages.set(i, message);
                env.storage().instance().set(&messages_key, &messages);

                env.events().publish(
                    (Symbol::new(env, "message_moderated"),),
                    (message_id, group_id, approved),
                );

                return Ok(());
            }
        }

        Err(Error::InvalidInput)
    }

    pub fn get_available_groups(env: &Env) -> Vec<PeerGroup> {
        // In a real implementation, this would query all groups
        // For now, return empty vec as we don't have a way to enumerate all groups
        Vec::new(env)
    }

    pub fn update_group_rules(
        env: &Env,
        group_id: String,
        moderator: Address,
        rules: Vec<String>,
    ) -> Result<(), Error> {
        let mut group: PeerGroup = env
            .storage()
            .instance()
            .get(&group_id)
            .ok_or(Error::GroupNotFound)?;

        if moderator != group.moderator {
            return Err(Error::Unauthorized);
        }

        group.rules = rules;
        env.storage().instance().set(&group_id, &group);

        env.events()
            .publish((Symbol::new(env, "group_rules_updated"),), group_id);

        Ok(())
    }

    fn contains_crisis_keywords(env: &Env, message_type: MessageType, content: String) -> bool {
        if message_type == MessageType::CrisisAlert {
            return true;
        }

        let content_lower = content.to_lowercase();

        // Crisis keywords
        let crisis_words = [
            "suicide",
            "kill myself",
            "end it",
            "not worth living",
            "self harm",
            "cut myself",
            "hurt myself",
            "overdose",
            "crisis",
            "emergency",
            "help me",
        ];

        for word in crisis_words.iter() {
            if content_lower.contains(word) {
                return true;
            }
        }

        false
    }

    fn flag_crisis_message(env: &Env, message_id: u64, group_id: String, sender: Address) {
        env.events().publish(
            (Symbol::new(env, "crisis_message_flagged"),),
            (message_id, group_id, sender),
        );
    }
}
