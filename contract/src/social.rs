use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec, symbol_short};
use crate::errors::ContractError;

#[derive(Clone)]
#[contracttype]
pub struct UserRating {
    pub user: Address,
    pub total_score: u64,
    pub review_count: u64,
    pub reputation_score: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Review {
    pub id: u64,
    pub reviewer: Address,
    pub reviewee: Address,
    pub trade_id: u64,
    pub rating: u32,
    pub comment: String,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct UserProfile {
    pub address: Address,
    pub followers: u64,
    pub following: u64,
    pub reputation: u64,
}

// Storage keys
fn rating_key(user: &Address) -> (soroban_sdk::Symbol, Address) {
    (symbol_short!("RATING"), user.clone())
}

fn review_key(review_id: u64) -> (soroban_sdk::Symbol, u64) {
    (symbol_short!("REVIEW"), review_id)
}

fn review_counter_key() -> soroban_sdk::Symbol {
    symbol_short!("REV_CNT")
}

fn follower_key(user: &Address, follower: &Address) -> (soroban_sdk::Symbol, Address, Address) {
    (symbol_short!("FOLLOW"), user.clone(), follower.clone())
}

fn profile_key(user: &Address) -> (soroban_sdk::Symbol, Address) {
    (symbol_short!("PROFILE"), user.clone())
}

pub fn submit_review(
    env: &Env,
    reviewer: &Address,
    reviewee: &Address,
    trade_id: u64,
    rating: u32,
    comment: String,
) -> Result<u64, ContractError> {
    reviewer.require_auth();

    if rating > 5 {
        return Err(ContractError::InvalidRating);
    }

    let review_id: u64 = env.storage().instance().get(&review_counter_key()).unwrap_or(0);
    let new_review_id = review_id + 1;

    let review = Review {
        id: new_review_id,
        reviewer: reviewer.clone(),
        reviewee: reviewee.clone(),
        trade_id,
        rating,
        comment,
        timestamp: env.ledger().timestamp(),
    };

    env.storage().instance().set(&review_key(new_review_id), &review);
    env.storage().instance().set(&review_counter_key(), &new_review_id);

    update_rating(env, reviewee, rating as u64)?;

    Ok(new_review_id)
}

pub fn update_rating(env: &Env, user: &Address, new_rating: u64) -> Result<(), ContractError> {
    let mut rating: UserRating = env
        .storage()
        .instance()
        .get(&rating_key(user))
        .unwrap_or(UserRating {
            user: user.clone(),
            total_score: 0,
            review_count: 0,
            reputation_score: 0,
        });

    rating.total_score += new_rating;
    rating.review_count += 1;
    rating.reputation_score = if rating.review_count > 0 {
        rating.total_score / rating.review_count
    } else {
        0
    };

    env.storage().instance().set(&rating_key(user), &rating);
    Ok(())
}

pub fn get_rating(env: &Env, user: &Address) -> Option<UserRating> {
    env.storage().instance().get(&rating_key(user))
}

pub fn get_review(env: &Env, review_id: u64) -> Option<Review> {
    env.storage().instance().get(&review_key(review_id))
}

pub fn follow_user(env: &Env, follower: &Address, user: &Address) -> Result<(), ContractError> {
    follower.require_auth();

    if follower == user {
        return Err(ContractError::CannotFollowSelf);
    }

    env.storage()
        .instance()
        .set(&follower_key(user, follower), &true);

    let mut user_profile = get_or_create_profile(env, user);
    user_profile.followers += 1;
    env.storage().instance().set(&profile_key(user), &user_profile);

    let mut follower_profile = get_or_create_profile(env, follower);
    follower_profile.following += 1;
    env.storage()
        .instance()
        .set(&profile_key(follower), &follower_profile);

    Ok(())
}

pub fn unfollow_user(env: &Env, follower: &Address, user: &Address) -> Result<(), ContractError> {
    follower.require_auth();

    let is_following: Option<bool> = env.storage().instance().get(&follower_key(user, follower));
    if is_following.is_none() {
        return Err(ContractError::NotFollowing);
    }

    env.storage().instance().remove(&follower_key(user, follower));

    let mut user_profile = get_or_create_profile(env, user);
    if user_profile.followers > 0 {
        user_profile.followers -= 1;
    }
    env.storage().instance().set(&profile_key(user), &user_profile);

    let mut follower_profile = get_or_create_profile(env, follower);
    if follower_profile.following > 0 {
        follower_profile.following -= 1;
    }
    env.storage()
        .instance()
        .set(&profile_key(follower), &follower_profile);

    Ok(())
}

pub fn get_profile(env: &Env, user: &Address) -> Option<UserProfile> {
    env.storage().instance().get(&profile_key(user))
}

fn get_or_create_profile(env: &Env, user: &Address) -> UserProfile {
    env.storage()
        .instance()
        .get(&profile_key(user))
        .unwrap_or(UserProfile {
            address: user.clone(),
            followers: 0,
            following: 0,
            reputation: 0,
        })
}

pub fn is_following(env: &Env, follower: &Address, user: &Address) -> bool {
    env.storage()
        .instance()
        .get(&follower_key(user, follower))
        .unwrap_or(false)
}
