# Patient Gamification System

## Overview

The Patient Gamification System is a comprehensive smart contract designed to increase patient engagement and encourage healthy behaviors through gamification elements. The system includes achievements, health challenges, reward points, social features, and leaderboards while maintaining strict privacy controls.

## Key Features

### 1. Achievement and Badge System

The achievement system rewards patients for completing health-related milestones and activities.

**Features:**
- Customizable achievements with configurable point rewards
- Badge URIs for visual representation
- Progress tracking towards achievement completion
- Automatic point awards upon achievement completion
- Achievement categories for organization

**Example Achievements:**
- "First Steps" - Record your first health metric
- "Week Warrior" - Maintain a 7-day activity streak
- "Health Champion" - Complete 10 health challenges
- "Data Guardian" - Maintain privacy settings for 30 days

### 2. Health Challenges and Competitions

Challenges create time-bound competitions that encourage patients to reach specific health goals.

**Features:**
- Time-bound challenges with start and end dates
- Target-based metrics (steps, calories, sleep hours, etc.)
- Participant limits to manage competition size
- Progress tracking during challenges
- Automatic completion detection and reward distribution

**Challenge Types:**
- Individual challenges (personal goals)
- Community challenges (group participation)
- Competitive challenges (leaderboard-based)
- Collaborative challenges (team-based goals)

### 3. Reward Points System

A flexible point system that rewards various patient activities and achievements.

**Features:**
- Configurable point values for different activities
- Total, available, and lifetime point tracking
- Point redemption capabilities
- Daily point limits to prevent gaming
- Automatic point awards for achievements and challenges

**Point Earning Activities:**
- Completing achievements
- Finishing challenges
- Maintaining daily streaks
- Recording health metrics
- Participating in social features

### 4. Social Features and Leaderboards

Privacy-preserving social features that encourage community engagement while protecting patient data.

**Features:**
- Optional public profiles with customizable privacy settings
- Leaderboards showing top performers
- Achievement and challenge visibility controls
- Display name and bio customization
- Avatar support

**Privacy Controls:**
- Public/private profile toggle
- Granular visibility for achievements, challenges, and points
- Anonymous participation options
- Data minimization principles

### 5. Health Metrics Tracking

Integration with existing patient data systems to track various health metrics.

**Supported Metrics:**
- Daily steps
- Blood pressure
- Heart rate
- Sleep duration
- Calories burned
- Water intake
- Medication adherence
- Custom metrics

**Features:**
- Historical data tracking
- Multiple data sources (manual, device, API)
- Unit conversion support
- Timestamp-based records

### 6. Daily Streaks

Encourages consistent engagement through streak tracking.

**Features:**
- Current and longest streak tracking
- Total active days counter
- Streak-based bonus points
- Automatic streak updates on activity

## Technical Architecture

### Data Structures

#### GamificationConfig
```rust
pub struct GamificationConfig {
    pub admin: Address,
    pub points_per_achievement: u32,
    pub points_per_challenge: u32,
    pub points_per_streak_day: u32,
    pub max_daily_points: u32,
    pub privacy_threshold: u32,
    pub enabled: bool,
}
```

#### Achievement
```rust
pub struct Achievement {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub points_reward: u32,
    pub badge_uri: String,
    pub requirement_type: String,
    pub requirement_value: u32,
    pub is_active: bool,
    pub created_at: u64,
}
```

#### HealthChallenge
```rust
pub struct HealthChallenge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub challenge_type: String,
    pub metric_name: String,
    pub target_value: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub points_reward: u32,
    pub max_participants: u32,
    pub current_participants: u32,
    pub is_active: bool,
    pub created_at: u64,
}
```

#### RewardPoints
```rust
pub struct RewardPoints {
    pub patient_id: Address,
    pub total_points: u64,
    pub available_points: u64,
    pub lifetime_points: u64,
    pub last_updated: u64,
}
```

#### LeaderboardEntry
```rust
pub struct LeaderboardEntry {
    pub patient_id: Address,
    pub display_name: String,
    pub points: u64,
    pub achievements_count: u32,
    pub challenges_completed: u32,
    pub rank: u32,
    pub last_updated: u64,
}
```

#### SocialProfile
```rust
pub struct SocialProfile {
    pub patient_id: Address,
    pub display_name: String,
    pub bio: String,
    pub avatar_uri: String,
    pub is_public: bool,
    pub show_achievements: bool,
    pub show_challenges: bool,
    pub show_points: bool,
    pub created_at: u64,
    pub last_active: u64,
}
```

## API Reference

### Initialization

#### `initialize`
Initializes the gamification contract with configuration parameters.

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    points_per_achievement: u32,
    points_per_challenge: u32,
    points_per_streak_day: u32,
    max_daily_points: u32,
    privacy_threshold: u32,
) -> Result<bool, Error>
```

**Parameters:**
- `admin`: Administrator address
- `points_per_achievement`: Points awarded per achievement
- `points_per_challenge`: Points awarded per challenge
- `points_per_streak_day`: Points per streak day
- `max_daily_points`: Maximum points earnable per day
- `privacy_threshold`: Minimum cohort size for privacy

### Achievement Management

#### `create_achievement`
Creates a new achievement that patients can earn.

```rust
pub fn create_achievement(
    env: Env,
    caller: Address,
    name: String,
    description: String,
    category: String,
    points_reward: u32,
    badge_uri: String,
    requirement_type: String,
    requirement_value: u32,
) -> Result<u64, Error>
```

#### `update_achievement_progress`
Updates a patient's progress towards an achievement.

```rust
pub fn update_achievement_progress(
    env: Env,
    caller: Address,
    patient_id: Address,
    achievement_id: u64,
    progress: u32,
) -> Result<bool, Error>
```

**Returns:** `true` if achievement was completed

#### `get_achievement`
Retrieves achievement details.

```rust
pub fn get_achievement(env: Env, achievement_id: u64) -> Result<Achievement, Error>
```

#### `get_patient_achievement`
Retrieves a patient's progress on a specific achievement.

```rust
pub fn get_patient_achievement(
    env: Env,
    patient_id: Address,
    achievement_id: u64,
) -> Result<PatientAchievement, Error>
```

#### `get_patient_achievements`
Retrieves all achievement IDs earned by a patient.

```rust
pub fn get_patient_achievements(env: Env, patient_id: Address) -> Result<Vec<u64>, Error>
```

### Challenge Management

#### `create_challenge`
Creates a new health challenge.

```rust
pub fn create_challenge(
    env: Env,
    caller: Address,
    name: String,
    description: String,
    challenge_type: String,
    metric_name: String,
    target_value: u32,
    start_time: u64,
    end_time: u64,
    points_reward: u32,
    max_participants: u32,
) -> Result<u64, Error>
```

#### `join_challenge`
Allows a patient to join a challenge.

```rust
pub fn join_challenge(
    env: Env,
    patient_id: Address,
    challenge_id: u64,
) -> Result<bool, Error>
```

#### `update_challenge_progress`
Updates a patient's progress in a challenge.

```rust
pub fn update_challenge_progress(
    env: Env,
    caller: Address,
    patient_id: Address,
    challenge_id: u64,
    value: u32,
) -> Result<bool, Error>
```

**Returns:** `true` if challenge was completed

#### `get_challenge`
Retrieves challenge details.

```rust
pub fn get_challenge(env: Env, challenge_id: u64) -> Result<HealthChallenge, Error>
```

#### `get_challenge_participant`
Retrieves a patient's participation in a challenge.

```rust
pub fn get_challenge_participant(
    env: Env,
    challenge_id: u64,
    patient_id: Address,
) -> Result<ChallengeParticipant, Error>
```

#### `get_challenge_participants`
Retrieves all participants in a challenge.

```rust
pub fn get_challenge_participants(
    env: Env,
    challenge_id: u64,
) -> Result<Vec<Address>, Error>
```

### Reward Points

#### `get_reward_points`
Retrieves a patient's point balance.

```rust
pub fn get_reward_points(env: Env, patient_id: Address) -> Result<RewardPoints, Error>
```

#### `redeem_points`
Allows a patient to redeem points.

```rust
pub fn redeem_points(
    env: Env,
    patient_id: Address,
    points: u64,
) -> Result<bool, Error>
```

### Social Features

#### `create_social_profile`
Creates a social profile for a patient.

```rust
pub fn create_social_profile(
    env: Env,
    patient_id: Address,
    display_name: String,
    bio: String,
    avatar_uri: String,
    is_public: bool,
    show_achievements: bool,
    show_challenges: bool,
    show_points: bool,
) -> Result<bool, Error>
```

#### `get_social_profile`
Retrieves a patient's social profile.

```rust
pub fn get_social_profile(env: Env, patient_id: Address) -> Result<SocialProfile, Error>
```

#### `update_social_profile`
Updates a patient's social profile.

```rust
pub fn update_social_profile(
    env: Env,
    patient_id: Address,
    display_name: String,
    bio: String,
    avatar_uri: String,
    is_public: bool,
    show_achievements: bool,
    show_challenges: bool,
    show_points: bool,
) -> Result<bool, Error>
```

### Leaderboard

#### `get_leaderboard`
Retrieves the top leaderboard entries.

```rust
pub fn get_leaderboard(env: Env, limit: u32) -> Result<Vec<LeaderboardEntry>, Error>
```

#### `get_patient_rank`
Retrieves a patient's rank on the leaderboard.

```rust
pub fn get_patient_rank(env: Env, patient_id: Address) -> Result<u32, Error>
```

### Health Metrics

#### `record_health_metric`
Records a health metric for a patient.

```rust
pub fn record_health_metric(
    env: Env,
    caller: Address,
    patient_id: Address,
    metric_name: String,
    value: u32,
    unit: String,
    source: String,
) -> Result<bool, Error>
```

#### `get_health_metric`
Retrieves a specific health metric record.

```rust
pub fn get_health_metric(
    env: Env,
    patient_id: Address,
    metric_name: String,
    timestamp: u64,
) -> Result<HealthMetric, Error>
```

#### `get_health_metrics`
Retrieves all timestamps for a specific metric.

```rust
pub fn get_health_metrics(
    env: Env,
    patient_id: Address,
    metric_name: String,
) -> Result<Vec<u64>, Error>
```

### Daily Streaks

#### `get_daily_streak`
Retrieves a patient's daily streak information.

```rust
pub fn get_daily_streak(env: Env, patient_id: Address) -> Result<DailyStreak, Error>
```

### Admin Functions

#### `update_config`
Updates the gamification configuration.

```rust
pub fn update_config(
    env: Env,
    caller: Address,
    points_per_achievement: u32,
    points_per_challenge: u32,
    points_per_streak_day: u32,
    max_daily_points: u32,
    privacy_threshold: u32,
    enabled: bool,
) -> Result<bool, Error>
```

#### `deactivate_achievement`
Deactivates an achievement.

```rust
pub fn deactivate_achievement(
    env: Env,
    caller: Address,
    achievement_id: u64,
) -> Result<bool, Error>
```

#### `deactivate_challenge`
Deactivates a challenge.

```rust
pub fn deactivate_challenge(
    env: Env,
    caller: Address,
    challenge_id: u64,
) -> Result<bool, Error>
```

### View Functions

#### `get_config`
Retrieves the current configuration.

```rust
pub fn get_config(env: Env) -> Result<GamificationConfig, Error>
```

#### `get_total_achievements`
Retrieves the total number of achievements created.

```rust
pub fn get_total_achievements(env: Env) -> Result<u64, Error>
```

#### `get_total_challenges`
Retrieves the total number of challenges created.

```rust
pub fn get_total_challenges(env: Env) -> Result<u64, Error>
```

## Privacy and Security

### Privacy-Preserving Features

1. **Optional Public Profiles**: Patients can choose to keep their profiles private
2. **Granular Visibility Controls**: Separate controls for achievements, challenges, and points
3. **Anonymous Participation**: Patients can participate without revealing identity
4. **Data Minimization**: Only necessary data is stored on-chain
5. **Privacy Thresholds**: Minimum cohort sizes for aggregated data

### Security Considerations

1. **Authentication**: All sensitive operations require patient authentication
2. **Authorization**: Admin functions are restricted to authorized addresses
3. **Input Validation**: All inputs are validated before processing
4. **Immutable Records**: Achievement and challenge records are immutable once created
5. **Event Logging**: All significant actions emit events for auditability

## Integration Guide

### Integrating with Existing Patient Data

The gamification system is designed to integrate seamlessly with existing patient data systems:

1. **Health Metrics**: Record metrics from existing health tracking systems
2. **Achievement Progress**: Update progress based on external data sources
3. **Challenge Participation**: Sync challenge data with existing patient portals
4. **Point Awards**: Award points for activities tracked in other systems

### Example Integration Flow

```rust
// 1. Patient records health metric in existing system
// 2. Existing system calls gamification contract
client.record_health_metric(
    &system_address,
    &patient_id,
    &String::from_str(&env, "steps"),
    &10000u32,
    &String::from_str(&env, "steps"),
    &String::from_str(&env, "fitbit"),
);

// 3. Check if achievement was completed
let achievement_completed = client.update_achievement_progress(
    &system_address,
    &patient_id,
    &achievement_id,
    &progress,
);

// 4. If completed, points are automatically awarded
// 5. Leaderboard is automatically updated
```

## Deployment

### Prerequisites

- Rust 1.78.0+
- Soroban CLI v23.1.4+
- Stellar account with XLM for deployment

### Build

```bash
cd contracts/patient_gamification
cargo build --target wasm32-unknown-unknown --release
```

### Deploy

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/patient_gamification.wasm \
  --source <ADMIN_SECRET_KEY> \
  --network <NETWORK>
```

### Initialize

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET_KEY> \
  --network <NETWORK> \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --points_per_achievement 100 \
  --points_per_challenge 50 \
  --points_per_streak_day 10 \
  --max_daily_points 1000 \
  --privacy_threshold 5
```

## Best Practices

### For Administrators

1. **Start Small**: Begin with a few achievements and challenges
2. **Monitor Engagement**: Track participation and adjust rewards accordingly
3. **Respect Privacy**: Always prioritize patient privacy over engagement metrics
4. **Regular Updates**: Keep challenges fresh and relevant
5. **Community Feedback**: Listen to patient feedback on gamification elements

### For Developers

1. **Error Handling**: Always handle errors gracefully
2. **Gas Optimization**: Minimize on-chain storage and computations
3. **Testing**: Thoroughly test all contract functions
4. **Documentation**: Keep documentation up-to-date
5. **Security Audits**: Regular security audits of contract code

### For Patients

1. **Privacy First**: Configure privacy settings before participating
2. **Regular Engagement**: Maintain daily streaks for bonus points
3. **Challenge Yourself**: Participate in challenges to earn more points
4. **Social Features**: Connect with others while maintaining privacy
5. **Track Progress**: Monitor achievements and leaderboard position

## Troubleshooting

### Common Issues

**Issue**: Achievement not completing
- **Solution**: Check that progress meets requirement_value
- **Solution**: Verify achievement is active

**Issue**: Cannot join challenge
- **Solution**: Check challenge is still active and within time range
- **Solution**: Verify challenge has not reached max participants

**Issue**: Points not awarded
- **Solution**: Check daily point limit has not been reached
- **Solution**: Verify achievement/challenge was actually completed

**Issue**: Leaderboard not updating
- **Solution**: Leaderboard updates automatically on point awards
- **Solution**: Check that patient has a social profile

## Support

For technical support or questions:
- Review the test files in `contracts/patient_gamification/src/test.rs`
- Check the main README.md for project-wide documentation
- Consult the Stellar Soroban documentation for platform-specific issues

## License

This project is part of the Stellar Uzima healthcare platform and follows the same licensing terms.
