// NATS Identity Projections
//
// Projects domain compositions (Organization × Person × Permissions)
// into NATS NKeys and JWT tokens.
//
// Functor chain:
//   Organization → NKey (Operator) → OperatorClaims → JWT
//   OrganizationUnit → NKey (Account) → AccountClaims → JWT (signed by Operator)
//   Person → NKey (User) → UserClaims → JWT (signed by Account)
//
// Each projection step emits events for audit trail.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    AccountIdentity, Organization, OrganizationUnit, Person, ServiceAccount, UserIdentity,
};
use crate::value_objects::{
    AccountClaims, AccountData, AccountLimits, NatsCredential, NatsJwt, NKeyPair, NKeyPublic,
    NKeySeed, NKeyType, OperatorClaims, OperatorData, Permissions, UserClaims, UserData,
    UserLimits,
};

// ============================================================================
// NKeys Integration
// ============================================================================

/// Convert our NKeyType to nkeys::KeyPairType
impl From<NKeyType> for nkeys::KeyPairType {
    fn from(key_type: NKeyType) -> Self {
        match key_type {
            NKeyType::Operator => nkeys::KeyPairType::Operator,
            NKeyType::Account => nkeys::KeyPairType::Account,
            NKeyType::User => nkeys::KeyPairType::User,
            NKeyType::Server => nkeys::KeyPairType::Server,
            NKeyType::Cluster => nkeys::KeyPairType::Cluster,
        }
    }
}

// ============================================================================
// NKey Generation Projections
// ============================================================================

/// NKey generation parameters
#[derive(Debug, Clone)]
pub struct NKeyGenerationParams {
    pub key_type: NKeyType,
    pub name: String,
    pub description: Option<String>,
    pub expires_after_days: Option<i64>,
}

/// Projection functor: Domain → NKeys
pub struct NKeyProjection;

impl NKeyProjection {
    /// Project organization to Operator NKey
    ///
    /// Creates the root authority key for the organization's NATS infrastructure.
    /// This key can sign Account JWTs.
    ///
    /// Emits: OperatorNKeyGeneratedEvent
    pub fn project_operator_nkey(organization: &Organization) -> NKeyGenerationParams {
        NKeyGenerationParams {
            key_type: NKeyType::Operator,
            name: format!("{} Operator", organization.name),
            description: Some(format!(
                "Root operator key for {} NATS infrastructure",
                organization.name
            )),
            expires_after_days: None, // Operators typically don't expire
        }
    }

    /// Project organization unit to Account NKey
    ///
    /// Creates an account key for a department/team/project.
    /// This key can sign User JWTs.
    ///
    /// Emits: AccountNKeyGeneratedEvent
    pub fn project_account_nkey(
        organization: &Organization,
        unit: &OrganizationUnit,
    ) -> NKeyGenerationParams {
        NKeyGenerationParams {
            key_type: NKeyType::Account,
            name: format!("{} - {} Account", organization.name, unit.name),
            description: Some(format!(
                "Account key for {} in {}",
                unit.name, organization.name
            )),
            expires_after_days: Some(365), // Rotate annually
        }
    }

    /// Project person to User NKey
    ///
    /// Creates a user key for individual authentication.
    ///
    /// Emits: UserNKeyGeneratedEvent
    pub fn project_user_nkey(person: &Person, organization: &Organization) -> NKeyGenerationParams {
        NKeyGenerationParams {
            key_type: NKeyType::User,
            name: format!("{} ({}) User", person.name, organization.name),
            description: Some(format!("User key for {}", person.email)),
            expires_after_days: Some(90), // Rotate quarterly
        }
    }

    /// Project UserIdentity to User NKey
    ///
    /// Unified projection supporting Person, Agent, or ServiceAccount.
    ///
    /// Emits: UserNKeyGeneratedEvent
    pub fn project_user_identity_nkey(
        user: &UserIdentity,
        organization: &Organization,
    ) -> NKeyGenerationParams {
        match user {
            UserIdentity::Person(person) => Self::project_user_nkey(person, organization),

            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(agent) => NKeyGenerationParams {
                key_type: NKeyType::User,
                name: format!("{} ({}) Agent", agent.name, organization.name),
                description: Some(format!("Agent key for {} - {}", agent.name, agent.agent_type)),
                expires_after_days: Some(180), // Agents rotate semi-annually
            },

            UserIdentity::ServiceAccount(service) => NKeyGenerationParams {
                key_type: NKeyType::User,
                name: format!("{} ({}) Service", service.name, organization.name),
                description: Some(format!("Service account key for {}", service.purpose)),
                expires_after_days: Some(365), // Services rotate annually
            },
        }
    }

    /// Project AccountIdentity to Account NKey
    ///
    /// Unified projection supporting Organization or OrganizationUnit.
    ///
    /// Emits: AccountNKeyGeneratedEvent
    pub fn project_account_identity_nkey(
        account: &AccountIdentity,
        parent_org: Option<&Organization>,
    ) -> NKeyGenerationParams {
        match account {
            AccountIdentity::Organization(org) => NKeyGenerationParams {
                key_type: NKeyType::Account,
                name: format!("{} Account", org.name),
                description: Some(format!("Organization account for {}", org.name)),
                expires_after_days: None, // Top-level org accounts don't expire
            },

            AccountIdentity::OrganizationUnit(unit) => {
                let org_name = parent_org.map(|o| o.name.as_str()).unwrap_or("Unknown");
                NKeyGenerationParams {
                    key_type: NKeyType::Account,
                    name: format!("{} - {} Account", org_name, unit.name),
                    description: Some(format!(
                        "{:?} account for {} in {}",
                        unit.unit_type,
                        unit.name,
                        org_name
                    )),
                    expires_after_days: Some(365), // Unit accounts rotate annually
                }
            }
        }
    }

    /// Generate NKey pair from parameters
    ///
    /// Uses the nkeys crate to generate real Ed25519 key pairs with proper NATS encoding.
    ///
    /// # Panics
    ///
    /// Panics if key generation fails (should never happen in practice)
    pub fn generate_nkey(params: &NKeyGenerationParams) -> NKeyPair {
        // Generate real Ed25519 key pair using nkeys crate
        let kp = nkeys::KeyPair::new(params.key_type.into());

        // Extract seed and public key
        let seed_string = kp.seed().expect("Failed to extract seed from KeyPair");
        let public_key_string = kp.public_key();

        // Wrap in our domain value objects
        let seed = NKeySeed::new(
            params.key_type,
            seed_string,
            Utc::now(),
        );

        let public_key = NKeyPublic::new(
            params.key_type,
            public_key_string,
        );

        // Create NKeyPair with optional expiration
        let mut nkey = NKeyPair::new(
            params.key_type,
            seed,
            public_key,
            Some(params.name.clone())
        );

        if let Some(days) = params.expires_after_days {
            let expiration = Utc::now() + Duration::days(days);
            nkey = nkey.with_expiration(expiration);
        }

        nkey
    }
}

// ============================================================================
// JWT Claims Projections
// ============================================================================

/// Projection functor: Domain + NKeys → JWT Claims
pub struct JwtClaimsProjection;

impl JwtClaimsProjection {
    /// Project organization + operator NKey to Operator JWT claims
    ///
    /// Creates self-signed operator claims.
    /// The operator key signs its own JWT.
    ///
    /// Emits: OperatorJwtClaimsCreatedEvent
    pub fn project_operator_claims(
        organization: &Organization,
        operator_nkey: &NKeyPair,
        signing_keys: Vec<String>,
    ) -> OperatorClaims {
        let now = Utc::now().timestamp();
        let public_key = operator_nkey.public_key_string();

        OperatorClaims {
            jti: Uuid::now_v7().to_string(),
            iat: now,
            iss: public_key.to_string(), // Self-signed
            sub: public_key.to_string(),
            nats: OperatorData {
                name: organization.name.clone(),
                signing_keys,
                version: 2,
                account_server_url: organization.metadata.get("nats_account_server").cloned(),
                operator_service_urls: organization
                    .metadata
                    .get("nats_service_urls")
                    .and_then(|urls| serde_json::from_str(urls).ok()),
            },
        }
    }

    /// Project org unit + account NKey to Account JWT claims
    ///
    /// Creates account claims signed by operator.
    ///
    /// Emits: AccountJwtClaimsCreatedEvent
    pub fn project_account_claims(
        organization: &Organization,
        unit: &OrganizationUnit,
        account_nkey: &NKeyPair,
        operator_nkey: &NKeyPair,
        signing_keys: Vec<String>,
        limits: Option<AccountLimits>,
    ) -> AccountClaims {
        let now = Utc::now().timestamp();

        AccountClaims {
            jti: Uuid::now_v7().to_string(),
            iat: now,
            iss: operator_nkey.public_key_string().to_string(), // Signed by operator
            sub: account_nkey.public_key_string().to_string(),
            exp: account_nkey.expires_at.map(|dt| dt.timestamp()),
            nats: AccountData {
                name: format!("{} - {}", organization.name, unit.name),
                signing_keys,
                version: 2,
                limits,
                default_permissions: Some(Self::default_account_permissions()),
            },
        }
    }

    /// Project person + user NKey to User JWT claims
    ///
    /// Creates user claims signed by account.
    ///
    /// Emits: UserJwtClaimsCreatedEvent
    pub fn project_user_claims(
        person: &Person,
        user_nkey: &NKeyPair,
        account_nkey: &NKeyPair,
        permissions: Option<Permissions>,
        limits: Option<UserLimits>,
    ) -> UserClaims {
        let now = Utc::now().timestamp();

        UserClaims {
            jti: Uuid::now_v7().to_string(),
            iat: now,
            iss: account_nkey.public_key_string().to_string(), // Signed by account
            sub: user_nkey.public_key_string().to_string(),
            exp: user_nkey.expires_at.map(|dt| dt.timestamp()),
            nats: UserData {
                name: person.name.clone(),
                version: 2,
                permissions,
                limits,
            },
        }
    }

    /// Project UserIdentity + user NKey to User JWT claims
    ///
    /// Unified projection for Person, Agent, or ServiceAccount.
    ///
    /// Emits: UserJwtClaimsCreatedEvent
    pub fn project_user_identity_claims(
        user: &UserIdentity,
        user_nkey: &NKeyPair,
        account_nkey: &NKeyPair,
        permissions: Option<Permissions>,
        limits: Option<UserLimits>,
    ) -> UserClaims {
        let now = Utc::now().timestamp();
        let user_name = user.name().to_string();

        UserClaims {
            jti: Uuid::now_v7().to_string(),
            iat: now,
            iss: account_nkey.public_key_string().to_string(), // Signed by account
            sub: user_nkey.public_key_string().to_string(),
            exp: user_nkey.expires_at.map(|dt| dt.timestamp()),
            nats: UserData {
                name: user_name,
                version: 2,
                permissions,
                limits,
            },
        }
    }

    /// Default account permissions (allow all, deny nothing)
    fn default_account_permissions() -> Permissions {
        Permissions {
            pub_allow: Some(vec![">".to_string()]), // Allow all publishes
            pub_deny: None,
            sub_allow: Some(vec![">".to_string()]), // Allow all subscriptions
            sub_deny: None,
        }
    }

    /// Default user limits (reasonable defaults)
    pub fn default_user_limits() -> UserLimits {
        UserLimits {
            subs: 100,           // 100 subscriptions
            data: -1,            // Unlimited data
            payload: 1024 * 1024, // 1MB max payload
        }
    }
}

// ============================================================================
// JWT Signing Projection
// ============================================================================

/// Projection functor: Claims + Signing Key → Signed JWT
pub struct JwtSigningProjection;

impl JwtSigningProjection {
    /// Sign operator claims to create Operator JWT
    ///
    /// The operator signs its own JWT (self-signed).
    ///
    /// Emits: OperatorJwtSignedEvent
    pub fn sign_operator_jwt(
        claims: OperatorClaims,
        operator_nkey: &NKeyPair,
    ) -> NatsJwt {
        let jwt_token = Self::encode_and_sign_jwt(&claims, operator_nkey)
            .expect("Failed to sign operator JWT");

        NatsJwt::new(
            NKeyType::Operator,
            jwt_token,
            operator_nkey.public_key.clone(),
            operator_nkey.public_key.clone(), // Self-signed
            Utc::now(),
            None, // Operators don't expire
        )
    }

    /// Sign account claims to create Account JWT
    ///
    /// The operator signs the account JWT.
    ///
    /// Emits: AccountJwtSignedEvent
    pub fn sign_account_jwt(
        claims: AccountClaims,
        operator_nkey: &NKeyPair,
        account_public_key: &NKeyPublic,
    ) -> NatsJwt {
        let expiration = claims.exp.map(|ts| DateTime::from_timestamp(ts, 0).unwrap());

        let jwt_token = Self::encode_and_sign_jwt(&claims, operator_nkey)
            .expect("Failed to sign account JWT");

        NatsJwt::new(
            NKeyType::Account,
            jwt_token,
            operator_nkey.public_key.clone(),
            account_public_key.clone(),
            Utc::now(),
            expiration,
        )
    }

    /// Sign user claims to create User JWT
    ///
    /// The account signs the user JWT.
    ///
    /// Emits: UserJwtSignedEvent
    pub fn sign_user_jwt(
        claims: UserClaims,
        account_nkey: &NKeyPair,
        user_public_key: &NKeyPublic,
    ) -> NatsJwt {
        let expiration = claims.exp.map(|ts| DateTime::from_timestamp(ts, 0).unwrap());

        let jwt_token = Self::encode_and_sign_jwt(&claims, account_nkey)
            .expect("Failed to sign user JWT");

        NatsJwt::new(
            NKeyType::User,
            jwt_token,
            account_nkey.public_key.clone(),
            user_public_key.clone(),
            Utc::now(),
            expiration,
        )
    }

    /// Encode claims and sign with NKey to create complete JWT
    ///
    /// This is the real implementation that creates proper NATS JWTs:
    /// 1. Serialize claims to JSON
    /// 2. Base64url encode header and claims
    /// 3. Sign with NKey seed
    /// 4. Create JWT as header.claims.signature
    fn encode_and_sign_jwt<T: Serialize>(
        claims: &T,
        signing_key: &NKeyPair,
    ) -> Result<String, String> {
        use base64::{Engine as _, engine::general_purpose};

        // Create JWT header
        let header = serde_json::json!({
            "typ": "JWT",
            "alg": "ed25519"
        });

        // Serialize header and claims to JSON
        let header_json = serde_json::to_string(&header)
            .map_err(|e| format!("Failed to serialize header: {}", e))?;
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| format!("Failed to serialize claims: {}", e))?;

        // Base64url encode (no padding)
        let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let claims_b64 = general_purpose::URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // Create signing payload: header.claims
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // Sign with NKey
        let kp = nkeys::KeyPair::from_seed(signing_key.seed_string())
            .map_err(|e| format!("Failed to create KeyPair from seed: {}", e))?;

        let signature = kp.sign(signing_input.as_bytes())
            .map_err(|e| format!("Failed to sign JWT: {}", e))?;

        // Base64url encode signature
        let signature_b64 = general_purpose::URL_SAFE_NO_PAD.encode(&signature);

        // Create complete JWT: header.claims.signature
        Ok(format!("{}.{}.{}", header_b64, claims_b64, signature_b64))
    }
}

// ============================================================================
// Complete NATS Identity Projection
// ============================================================================

/// Complete NATS identity projection result
#[derive(Debug, Clone)]
pub struct NatsIdentityProjection {
    /// The NKey pair
    pub nkey: NKeyPair,
    /// The signed JWT
    pub jwt: NatsJwt,
    /// Combined credential (for user credentials)
    pub credential: Option<NatsCredential>,
}

/// High-level projection functions
pub struct NatsProjection;

impl NatsProjection {
    /// Complete projection: Organization → Operator (NKey + JWT)
    ///
    /// Creates complete operator identity.
    ///
    /// Emits:
    /// - OperatorNKeyGeneratedEvent
    /// - OperatorJwtClaimsCreatedEvent
    /// - OperatorJwtSignedEvent
    pub fn project_operator(organization: &Organization) -> NatsIdentityProjection {
        // Generate NKey
        let params = NKeyProjection::project_operator_nkey(organization);
        let nkey = NKeyProjection::generate_nkey(&params);

        // Create claims
        let signing_keys = vec![]; // TODO: Load from org metadata
        let claims = JwtClaimsProjection::project_operator_claims(organization, &nkey, signing_keys);

        // Sign JWT
        let jwt = JwtSigningProjection::sign_operator_jwt(claims, &nkey);

        NatsIdentityProjection {
            nkey,
            jwt,
            credential: None, // Operators don't need credential files
        }
    }

    /// Complete projection: OrgUnit → Account (NKey + JWT)
    ///
    /// Creates complete account identity signed by operator.
    ///
    /// Emits:
    /// - AccountNKeyGeneratedEvent
    /// - AccountJwtClaimsCreatedEvent
    /// - AccountJwtSignedEvent
    pub fn project_account(
        organization: &Organization,
        unit: &OrganizationUnit,
        operator_nkey: &NKeyPair,
        limits: Option<AccountLimits>,
    ) -> NatsIdentityProjection {
        // Generate NKey
        let params = NKeyProjection::project_account_nkey(organization, unit);
        let nkey = NKeyProjection::generate_nkey(&params);

        // Create claims
        let signing_keys = vec![]; // TODO: Load from unit metadata
        let claims = JwtClaimsProjection::project_account_claims(
            organization,
            unit,
            &nkey,
            operator_nkey,
            signing_keys,
            limits,
        );

        // Sign JWT (operator signs account)
        let jwt = JwtSigningProjection::sign_account_jwt(claims, operator_nkey, &nkey.public_key);

        NatsIdentityProjection {
            nkey,
            jwt,
            credential: None, // Accounts don't need credential files
        }
    }

    /// Complete projection: Person → User (NKey + JWT + Credential)
    ///
    /// Creates complete user identity signed by account.
    ///
    /// Emits:
    /// - UserNKeyGeneratedEvent
    /// - UserJwtClaimsCreatedEvent
    /// - UserJwtSignedEvent
    /// - UserCredentialCreatedEvent
    pub fn project_user(
        person: &Person,
        organization: &Organization,
        account_nkey: &NKeyPair,
        permissions: Option<Permissions>,
        limits: Option<UserLimits>,
    ) -> NatsIdentityProjection {
        // Generate NKey
        let params = NKeyProjection::project_user_nkey(person, organization);
        let nkey = NKeyProjection::generate_nkey(&params);

        // Create claims
        let claims = JwtClaimsProjection::project_user_claims(
            person,
            &nkey,
            account_nkey,
            permissions,
            limits,
        );

        // Sign JWT (account signs user)
        let jwt = JwtSigningProjection::sign_user_jwt(claims, account_nkey, &nkey.public_key);

        // Create credential file (JWT + seed combined)
        let credential = NatsCredential::new(
            jwt.clone(),
            nkey.seed.clone(),
            Some(person.name.clone()),
        );

        NatsIdentityProjection {
            nkey,
            jwt,
            credential: Some(credential),
        }
    }

    /// Complete projection: UserIdentity → User (NKey + JWT + Credential)
    ///
    /// UNIFIED projection supporting Person, Agent, or ServiceAccount.
    ///
    /// This is the recommended projection function for creating NATS users.
    ///
    /// Emits:
    /// - UserNKeyGeneratedEvent
    /// - UserJwtClaimsCreatedEvent
    /// - UserJwtSignedEvent
    /// - UserCredentialCreatedEvent
    pub fn project_user_identity(
        user: &UserIdentity,
        organization: &Organization,
        account_nkey: &NKeyPair,
        permissions: Option<Permissions>,
        limits: Option<UserLimits>,
    ) -> NatsIdentityProjection {
        // Generate NKey
        let params = NKeyProjection::project_user_identity_nkey(user, organization);
        let nkey = NKeyProjection::generate_nkey(&params);

        // Create claims
        let claims = JwtClaimsProjection::project_user_identity_claims(
            user,
            &nkey,
            account_nkey,
            permissions,
            limits,
        );

        // Sign JWT (account signs user)
        let jwt = JwtSigningProjection::sign_user_jwt(claims, account_nkey, &nkey.public_key);

        // Create credential file (JWT + seed combined)
        let credential = NatsCredential::new(
            jwt.clone(),
            nkey.seed.clone(),
            Some(user.name().to_string()),
        );

        NatsIdentityProjection {
            nkey,
            jwt,
            credential: Some(credential),
        }
    }

    /// Complete projection: AccountIdentity → Account (NKey + JWT)
    ///
    /// UNIFIED projection supporting Organization or OrganizationUnit.
    ///
    /// This is the recommended projection function for creating NATS accounts.
    ///
    /// Emits:
    /// - AccountNKeyGeneratedEvent
    /// - AccountJwtClaimsCreatedEvent
    /// - AccountJwtSignedEvent
    pub fn project_account_identity(
        account: &AccountIdentity,
        parent_org: Option<&Organization>,
        operator_nkey: &NKeyPair,
        limits: Option<AccountLimits>,
    ) -> NatsIdentityProjection {
        // Generate NKey
        let params = NKeyProjection::project_account_identity_nkey(account, parent_org);
        let nkey = NKeyProjection::generate_nkey(&params);

        // Extract organization details
        let (org, unit) = match account {
            AccountIdentity::Organization(o) => (o, None),
            AccountIdentity::OrganizationUnit(u) => {
                (parent_org.expect("OrganizationUnit requires parent_org"), Some(u))
            }
        };

        // Create claims
        let signing_keys = vec![]; // TODO: Load from metadata
        let claims = if let Some(unit) = unit {
            JwtClaimsProjection::project_account_claims(
                org,
                unit,
                &nkey,
                operator_nkey,
                signing_keys,
                limits,
            )
        } else {
            // Organization as account - create synthetic unit
            let synthetic_unit = OrganizationUnit {
                id: org.id,
                name: org.name.clone(),
                unit_type: crate::domain::OrganizationUnitType::Infrastructure,
                parent_unit_id: None,
                responsible_person_id: None,
            };
            JwtClaimsProjection::project_account_claims(
                org,
                &synthetic_unit,
                &nkey,
                operator_nkey,
                signing_keys,
                limits,
            )
        };

        // Sign JWT (operator signs account)
        let jwt = JwtSigningProjection::sign_account_jwt(claims, operator_nkey, &nkey.public_key);

        NatsIdentityProjection {
            nkey,
            jwt,
            credential: None, // Accounts don't need credential files
        }
    }

    /// **Organization Bootstrap Projection**
    ///
    /// Complete functor mapping from Organization domain to full NATS infrastructure.
    ///
    /// This is the PRIMARY bootstrap function for creating a complete NATS hierarchy:
    /// - 1 Operator (Organization)
    /// - N Accounts (OrganizationUnits)
    /// - M Users (People across all units)
    ///
    /// **Input**: Organization with units + People
    /// **Output**: OrganizationBootstrap with all NATS identities
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let org = load_organization_from_json("domain-bootstrap.json")?;
    /// let people = load_people_from_json("people.json")?;
    ///
    /// let bootstrap = NatsProjection::bootstrap_organization(&org, &people);
    ///
    /// // Write operator JWT
    /// fs::write("operator.jwt", bootstrap.operator.jwt.token())?;
    ///
    /// // Write account JWTs
    /// for (unit_id, (unit, account)) in &bootstrap.accounts {
    ///     fs::write(format!("account-{}.jwt", unit_id), account.jwt.token())?;
    /// }
    ///
    /// // Write user credentials
    /// for (person_id, (person, user)) in &bootstrap.users {
    ///     if let Some(cred) = &user.credential {
    ///         fs::write(format!("user-{}.creds", person_id), cred.to_string())?;
    ///     }
    /// }
    /// ```
    ///
    /// # Events Emitted
    ///
    /// - 1 × OperatorIdentityCreatedEvent
    /// - N × AccountIdentityCreatedEvent (one per unit)
    /// - M × UserIdentityCreatedEvent (one per person)
    ///
    /// Total events: 1 + N + M (where N = units, M = people)
    pub fn bootstrap_organization(
        organization: &Organization,
        people: &[Person],
    ) -> OrganizationBootstrap {
        // Step 1: Create Operator identity (root of trust)
        let operator = Self::project_operator(organization);

        // Step 2: Create Account identities for each OrganizationUnit
        let mut accounts = std::collections::HashMap::new();
        for unit in &organization.units {
            let account = Self::project_account(organization, unit, &operator.nkey, None);
            accounts.insert(unit.id, (unit.clone(), account));
        }

        // Ensure we have at least one account (create default if needed)
        if accounts.is_empty() {
            let default_unit = OrganizationUnit {
                id: organization.id,
                name: format!("{} - Default", organization.name),
                unit_type: crate::domain::OrganizationUnitType::Infrastructure,
                parent_unit_id: None,
                responsible_person_id: None,
            };
            let default_account = Self::project_account(
                organization,
                &default_unit,
                &operator.nkey,
                None
            );
            accounts.insert(default_unit.id, (default_unit, default_account));
        }

        // Step 3: Create User identities for all people
        let mut users = std::collections::HashMap::new();

        for person in people {
            // Filter to only people belonging to this organization
            if person.organization_id != organization.id {
                continue;
            }

            // Determine which account this user belongs to based on their unit_ids
            let account_nkey = if let Some(unit_id) = person.unit_ids.first() {
                // User belongs to a specific unit - use that unit's account
                if let Some((_, account)) = accounts.get(unit_id) {
                    &account.nkey
                } else {
                    // Fallback to first available account
                    &accounts.iter().next().unwrap().1.1.nkey
                }
            } else {
                // No unit specified - use first available account
                &accounts.iter().next().unwrap().1.1.nkey
            };

            let user = Self::project_user(
                person,
                organization,
                account_nkey,
                None, // Default permissions
                None, // Default limits
            );

            users.insert(person.id, (person.clone(), user));
        }

        // Step 4: Create Service Account identities (if any)
        let service_accounts = std::collections::HashMap::new();
        // TODO: Extract service accounts from organization metadata when available

        OrganizationBootstrap {
            organization: organization.clone(),
            operator,
            accounts,
            users,
            service_accounts,
        }
    }
}

/// Complete organization NATS bootstrap result
///
/// Contains ALL NATS identities needed to operate a complete NATS infrastructure
/// for an organization.
///
/// This is the output of the primary bootstrap projection functor:
///   F: Organization → (Operator, [Account], [User])
///
/// **Category Theory Interpretation:**
/// - Source Category: Organization domain (entities and relationships)
/// - Target Category: NATS infrastructure (operators, accounts, users)
/// - Functor: NatsProjection::bootstrap_organization
/// - Preserves: Organizational hierarchy and trust relationships
#[derive(Debug, Clone)]
pub struct OrganizationBootstrap {
    /// The source organization
    pub organization: Organization,

    /// Operator identity (1 per organization)
    pub operator: NatsIdentityProjection,

    /// Account identities (1 per organizational unit)
    /// Key: OrganizationUnit ID
    pub accounts: std::collections::HashMap<Uuid, (OrganizationUnit, NatsIdentityProjection)>,

    /// User identities (1 per person)
    /// Key: Person ID
    pub users: std::collections::HashMap<Uuid, (Person, NatsIdentityProjection)>,

    /// Service account identities
    /// Key: ServiceAccount ID
    pub service_accounts: std::collections::HashMap<Uuid, (ServiceAccount, NatsIdentityProjection)>,
}

impl OrganizationBootstrap {
    /// Get total number of identities created
    pub fn total_identities(&self) -> usize {
        1 + self.accounts.len() + self.users.len() + self.service_accounts.len()
    }

    /// Get all NKey seeds (for secure backup)
    pub fn all_seeds(&self) -> Vec<&NKeySeed> {
        let mut seeds = vec![&self.operator.nkey.seed];

        for (_, (_, account)) in &self.accounts {
            seeds.push(&account.nkey.seed);
        }

        for (_, (_, user)) in &self.users {
            seeds.push(&user.nkey.seed);
        }

        for (_, (_, service)) in &self.service_accounts {
            seeds.push(&service.nkey.seed);
        }

        seeds
    }

    /// Get all credentials that need to be distributed
    pub fn all_credentials(&self) -> Vec<(&Uuid, &NatsCredential)> {
        let mut creds = Vec::new();

        for (person_id, (_, user)) in &self.users {
            if let Some(cred) = &user.credential {
                creds.push((person_id, cred));
            }
        }

        for (service_id, (_, service)) in &self.service_accounts {
            if let Some(cred) = &service.credential {
                creds.push((service_id, cred));
            }
        }

        creds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_nkey_projection() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let params = NKeyProjection::project_operator_nkey(&org);
        assert_eq!(params.key_type, NKeyType::Operator);
        assert!(params.name.contains("Test Org"));
        assert!(params.expires_after_days.is_none());
    }

    #[test]
    fn test_account_nkey_projection() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: crate::domain::OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
        };

        let params = NKeyProjection::project_account_nkey(&org, &unit);
        assert_eq!(params.key_type, NKeyType::Account);
        assert!(params.name.contains("Engineering"));
        assert_eq!(params.expires_after_days, Some(365));
    }

    #[test]
    fn test_nkey_generation() {
        let params = NKeyGenerationParams {
            key_type: NKeyType::User,
            name: "Test User".to_string(),
            description: None,
            expires_after_days: Some(90),
        };

        let nkey = NKeyProjection::generate_nkey(&params);
        assert_eq!(nkey.key_type, NKeyType::User);
        assert!(nkey.expires_at.is_some());

        // Verify the NKey has valid prefixes
        assert!(nkey.seed.is_valid_prefix(), "Seed should have valid prefix (SU...)");
        assert!(nkey.public_key.is_valid_prefix(), "Public key should have valid prefix (U...)");

        // Verify we can recreate KeyPair from seed
        let kp = nkeys::KeyPair::from_seed(nkey.seed_string());
        assert!(kp.is_ok(), "Should be able to recreate KeyPair from seed");

        let kp = kp.unwrap();
        assert_eq!(kp.public_key(), nkey.public_key_string(), "Public keys should match");
    }

    #[test]
    fn test_complete_operator_projection() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "CowboyAI".to_string(),
            display_name: "The Cowboy AI".to_string(),
            description: Some("Test organization for CIM infrastructure".to_string()),
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        // Project organization to complete operator identity
        let identity = NatsProjection::project_operator(&org);

        // Verify NKey
        assert_eq!(identity.nkey.key_type, NKeyType::Operator);
        assert!(identity.nkey.seed.is_valid_prefix());
        assert!(identity.nkey.public_key.is_valid_prefix());

        // Verify JWT token is properly formatted (header.claims.signature)
        let jwt_parts: Vec<&str> = identity.jwt.token().split('.').collect();
        assert_eq!(jwt_parts.len(), 3, "JWT should have 3 parts: header.claims.signature");

        // Verify we can verify the JWT signature
        let kp = nkeys::KeyPair::from_seed(identity.nkey.seed_string()).unwrap();
        let signing_input = format!("{}.{}", jwt_parts[0], jwt_parts[1]);

        use base64::{Engine as _, engine::general_purpose};
        let signature = general_purpose::URL_SAFE_NO_PAD.decode(jwt_parts[2]).unwrap();

        let verification = kp.verify(signing_input.as_bytes(), &signature);
        assert!(verification.is_ok(), "JWT signature should verify correctly");
    }

    #[test]
    fn test_account_jwt_signed_by_operator() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "CowboyAI".to_string(),
            display_name: "The Cowboy AI".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: crate::domain::OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
        };

        // Create operator
        let operator = NatsProjection::project_operator(&org);

        // Create account signed by operator
        let account = NatsProjection::project_account(&org, &unit, &operator.nkey, None);

        // Verify account JWT is signed by operator
        assert_eq!(account.jwt.issuer.public_key(), operator.nkey.public_key_string());
        assert_eq!(account.jwt.subject.public_key(), account.nkey.public_key_string());

        // Verify operator can verify the account JWT signature
        let jwt_parts: Vec<&str> = account.jwt.token().split('.').collect();
        let signing_input = format!("{}.{}", jwt_parts[0], jwt_parts[1]);

        use base64::{Engine as _, engine::general_purpose};
        let signature = general_purpose::URL_SAFE_NO_PAD.decode(jwt_parts[2]).unwrap();

        let operator_kp = nkeys::KeyPair::from_seed(operator.nkey.seed_string()).unwrap();
        let verification = operator_kp.verify(signing_input.as_bytes(), &signature);
        assert!(verification.is_ok(), "Account JWT should verify with operator's key");
    }
}
