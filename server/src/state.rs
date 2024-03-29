use std::collections::HashMap;

/// Represents a user's login credentials.
type Login = String;

/// Represents a user's password.
type Pass = String;

/// Represents the last hash of a user.
type LastHash = String;

/// Enumerates possible user states.
///
/// * `Auth`: Indicates that the user is authenticated.
/// * `InProcess`: Indicates that the user authentication is in progress.
#[derive(PartialEq, Debug)]
pub enum UserState {
    Auth,
    InProcess,
}

/// Represents the current state of the application.
///
/// Contains information about registered users, authenticated users,
/// available quotes, and the current difficulty level for Proof-of-Work.
#[derive(Debug)]
pub struct State {
    /// Map of registered users and their passwords.
    pub users: HashMap<Login, Pass>,

    /// Map of authenticated users, their last hash, and their authentication state.
    pub authorized: HashMap<Login, (LastHash, UserState)>,

    /// Collection of available quotes to be provided to authenticated users.
    pub quotes: Vec<String>,

    /// Difficulty level for Proof-of-Work.
    pub difficulty: u64,
}
impl State {
    /// Constructs a new `State` with predefined users and available quotes.
    ///
    /// * Returns
    ///
    /// A `State` object initialized with predefined values.
    pub fn new() -> State {
        // users already registered (better way is using session id or smth)
        let users = HashMap::from([
            (("one".to_string()), "pass1".to_string()),
            ("two".to_string(), "pass2".to_string()),
            ("three".to_string(), "pass3".to_string()),
        ]);

        let authorized = HashMap::new();

        let quotes = vec![
            r#"
            The ungodly ... reasoned unsoundly, saying to themselves,...
            we were born by mere chance,
            and hereafter we shall be as though we had never been,
            for the breath in our nostrils is smoke,
            and reason is a spark kindled by the beating of our hearts
            when it is extinguished, the body will turn to ashes,
            and the spirit will dissolve like empty air.
            "#
            .to_string(),
            r#"         
            The ungodly ... reasoned unsoundly, saying to themselves,...
            Come, therefore, let us enjoy the good things that exist,
            and make use of the creation to the full as in youth.
            Let us take our fill of costly wine and perfumes,
            and let no flower of spring pass us by.
            Let us crown ourselves with rosebuds before they wither.
            Let none of us fail to share in our revelry;
            everywhere let us leave signs of enjoyment,
            because this is our portion, and this our lot.
            Let us oppress the righteous poor man;
            let us not spare the widow
            or regard the gray hairs of the aged.
            But let our might be our law of right,
            for what is weak proves itself to be useless.
            "#
            .to_string(),
            r#"
            Thus they reasoned, but they were led astray, ...
            for God created us for incorruption,
            and made us in the image of his own eternity.
            "#
            .to_string(),
            r#"
            In the memory of virtue is immortality,
            because it is known both by God and by mortals.
            When it is present, people imitate it,
            and they long for it when it has gone;
            throughout all time it marches, crowned in triumph.
            "#
            .to_string(),
            r#"
            What has our arrogance profited us?
            And what good has our boasted wealth brought us?
            All those things have vanished like a shadow,
            and like a rumor that passes by;
            like a ship that sails through the billowy water,
            and when it has passed no trace can be found,
            no track of its keel in the waves.
            "#
            .to_string(),
        ];

        State {
            users,
            authorized,
            quotes,
            difficulty: 4,
        }
    }
}
