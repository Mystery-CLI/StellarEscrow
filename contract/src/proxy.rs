use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Val, Vec};

const STORAGE_ADMIN: Symbol = symbol_short!("ADMIN");
const STORAGE_LOGIC: Symbol = symbol_short!("LOGIC_ADDR");

#[contract]
pub struct EscrowProxy;

#[contractimpl]
impl EscrowProxy {
    /// Initialize the proxy with an admin and an initial logic contract.
    pub fn init_proxy(env: Env, admin: Address, logic_contract: Address) {
        if env.storage().instance().has(&STORAGE_ADMIN) {
            panic!("Already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&STORAGE_ADMIN, &admin);
        env.storage().instance().set(&STORAGE_LOGIC, &logic_contract);
    }

    /// Set the logic contract address
    pub fn set_logic_contract(env: Env, address: Address) {
        let admin: Address = env.storage().instance().get(&STORAGE_ADMIN).expect("Not initialized");
        admin.require_auth();
        env.storage().instance().set(&STORAGE_LOGIC, &address);
    }

    /// Get the current logic contract address
    pub fn get_logic_contract(env: Env) -> Address {
        env.storage().instance().get(&STORAGE_LOGIC).expect("Not initialized")
    }

    /// Upgrade function (CRITICAL)
    /// Updates the logic contract address, only callable by admin
    pub fn upgrade(env: Env, new_logic: Address) {
        let admin: Address = env.storage().instance().get(&STORAGE_ADMIN).expect("Not initialized");
        admin.require_auth();
        
        // Validate new address (just checking it's not the same currently, basic validation)
        let current_logic: Address = env.storage().instance().get(&STORAGE_LOGIC).unwrap();
        if current_logic == new_logic {
            panic!("Same address");
        }
        
        env.storage().instance().set(&STORAGE_LOGIC, &new_logic);
        
        // Emit upgrade event
        env.events().publish((symbol_short!("upgrade"),), new_logic);
    }

    /// Fallback method to forward all other calls to the logic contract.
    /// This delegates calls using env.invoke_contract(), preserving state in proxy (logic is stateless)
    pub fn delegate(env: Env, func: Symbol, args: Vec<Val>) -> Val {
        let logic_contract: Address = env.storage().instance().get(&STORAGE_LOGIC).expect("Not initialized");
        env.invoke_contract(&logic_contract, &func, args)
    }
}
