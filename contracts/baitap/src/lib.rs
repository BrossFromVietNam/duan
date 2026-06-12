#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

// Define the storage keys used in the Contract's Instance Storage
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PricePerMonth,
    RobotMaintenance,         // Maintenance status of the robot (bool)
    UserSubscription(Address), // Stores the subscription expiration timestamp (u64) for each user
}

#[contract]
pub struct OfficeCleaningRobotContract;

#[contractimpl]
impl OfficeCleaningRobotContract {
    
    /// Initializes the system configuration (Can only be called once)
    pub fn init(env: Env, admin: Address, price: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract has already been initialized!");
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PricePerMonth, &price);
        env.storage().instance().set(&DataKey::RobotMaintenance, &false); // Robot is active by default
    }

    /// Updates the robot's maintenance/repair status (Only the Admin has permission)
    pub fn set_maintenance(env: Env, admin: Address, maintenance: bool) {
        // 1. Verify the Admin's signature
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        
        if admin != stored_admin {
            panic!("Only the Admin can modify the robot maintenance status");
        }

        // 2. Update the maintenance status
        env.storage().instance().set(&DataKey::RobotMaintenance, &maintenance);
    }

    /// Users purchase or renew a monthly subscription (30 days = 2,592,000 seconds)
    pub fn subscribe_month(env: Env, user: Address, payment_amount: i128) {
        // 1. Verify the user's signature
        user.require_auth();

        // 2. CHECK: If the robot is undergoing repairs, suspend new registrations/renewals
        let is_maintenance: bool = env.storage().instance().get(&DataKey::RobotMaintenance).unwrap_or(false);
        if is_maintenance {
            panic!("The robot is currently under maintenance. Service is suspended, please try again later!");
        }

        // 3. CHECK: Verify if the payment matches the configured monthly price
        let price_per_month: i128 = env.storage().instance().get(&DataKey::PricePerMonth).unwrap();
        if payment_amount < price_per_month {
            panic!("Insufficient payment amount to subscribe for a monthly plan");
        }

        // 4. Calculate the new expiration timestamp
        let current_time = env.ledger().timestamp();
        let thirty_days_in_seconds: u64 = 30 * 24 * 60 * 60;

        let current_expiry: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UserSubscription(user.clone()))
            .unwrap_or(0);

        let new_expiry = if current_expiry > current_time {
            // Case 1: Active subscription -> Accumulate and add 30 days onto the existing expiration date
            current_expiry + thirty_days_in_seconds
        } else {
            // Case 2: New or expired subscription -> Add 30 days starting from the current block time
            current_time + thirty_days_in_seconds
        };

        // 5. Save the user's account account details to Storage
        env.storage().instance().set(&DataKey::UserSubscription(user), &new_expiry);
    }

    /// Access control check: The robot or office gate client calls this function before performing work
    pub fn check_access(env: Env, user: Address) -> bool {
        // SUSPENSION CHECK: If the robot is being repaired, deny service immediately (return false)
        let is_maintenance: bool = env.storage().instance().get(&DataKey::RobotMaintenance).unwrap_or(false);
        if is_maintenance {
            return false;
        }

        // Retrieve the user's subscription expiration time
        let expiry: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UserSubscription(user))
            .unwrap_or(0);
            
        let current_time = env.ledger().timestamp();

        // Returns true only if the subscription is still valid (expiry time > current ledger time)
        expiry > current_time
    }

    /// Returns the Unix Timestamp of a specific user's expiration date
    pub fn get_user_expiry(env: Env, user: Address) -> u64 {
        env.storage().instance().get(&DataKey::UserSubscription(user)).unwrap_or(0)
    }

    /// Checks whether the robot is currently under maintenance
    pub fn is_robot_repairing(env: Env) -> bool {
        env.storage().instance().get(&DataKey::RobotMaintenance).unwrap_or(false)
    }
}