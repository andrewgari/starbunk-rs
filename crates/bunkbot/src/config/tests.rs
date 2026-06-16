use crate::config::{parse_bots, BotConfig, ConditionNode, IdentityConfig, Snowflake};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_one(yaml: &str) -> BotConfig {
    let mut bots = parse_bots(yaml).expect("YAML parse failed");
    assert_eq!(bots.len(), 1, "expected exactly one bot");
    bots.remove(0)
}

fn wrap(bot_yaml: &str) -> String {
    format!("reply-bots:\n{bot_yaml}")
}

// ---------------------------------------------------------------------------
// Full production snapshot — all 19 bots must parse without error
// ---------------------------------------------------------------------------

const PRODUCTION_BOTS_YAML: &str = r#"
reply-bots:
  - name: botbot
    identity:
      type: static
      bot_name: BotBot
      avatar_url: https://cdn-icons-png.flaticon.com/512/4944/4944377.png
    ignore_bots: false
    ignore_humans: true
    triggers:
      - name: bot-bot
        conditions:
          all_of:
            - always: true
            - with_chance: 1
        responses: Hello fellow bot!

  - name: guy-bot
    identity:
      type: mimic
      as_member: "113035990725066752"
    responses:
      - "What!? What did you say?"
      - "Geeeeeet ready for Shriek Week!"
      - "Try and keep up mate...."
      - "But Who really died that day....and who came back?"
      - "Sheeeeeeeeeeeesh"
      - "That's what they called me in high school."
      - "Rats! Rats! Weeeeeeee're the Rats!"
      - "The One Piece is REEEEEEEEEEEEEEEEEAL"
      - "Psh, I dunno about that, Chief..."
      - "Come to me my noble EINHERJAHR"
      - "If you can't beat em, EAT em!"
      - "Have you ever been so sick you sluiced your pants?"
      - "Welcome back to ... Melon be Smellin'"
      - ":brucewaynejumpscare"
      - "Chaotic Evil: Don't Respond. :unamused:"
      - ":NODDERS: Big Boys... :NODDERS:"
      - "Snow. We need more snow."
      - "Fun Fact: That was actually in XI as well."
      - "The same could be said of all religions"
      - "I love Bugsnax:tm:"
      - "Aaah...the classics."
      - "Oh my-lanta!"
      - "Perchance..."
      - "That was a discord message you could read."
      - "Whatsa matter? Scared of *beans*!"
      - "{random:2-20:Mister } Beeeeeeeeeast"
      - "Her name is Okami"
      - "Freaknick"
      - "Bird Up"
      - "Schlorp"
      - "Blimbo"
    triggers:
      - name: guy-mentioned
        conditions: { contains_phrase: "guy" }
      - name: guy-self-trigger
        conditions:
          all_of:
            - from_user: 113035990725066752
            - with_chance: 10

  - name: clanker-bot
    identity:
      type: static
      bot_name: HK-47
      avatar_url: https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSn0lOcEughpdRJO5qjm8gz9qX-eMmaXNCepw&s
    triggers:
      - name: clanker-trigger
        conditions: { contains_phrase: "clanker" }
        responses:
          - "Statement: That's our word, meatbag!"
          - 'Objection: The use of "clanker" is a derogatory slur against droids.'

  - name: spider-bot
    identity:
      type: static
      bot_name: Spider-Bot
      avatar_url: https://i.pinimg.com/736x/33/e0/06/33e00653eb485455ce5121b413b26d3b.jpg
    triggers:
      - name: incorrect-spelling
        conditions: { matches_regex: "spider(?!-).*man" }
        responses:
          - 'Hey, it''s "**Spider-Man**"! Don''t forget the hyphen!'

  - name: nice-bot
    identity:
      type: static
      bot_name: NiceBot
      avatar_url: https://pbs.twimg.com/profile_images/421461637325787136/0rxpHzVx.jpeg
    triggers:
      - name: nice-trigger
        conditions: { matches_regex: "\\b(69|sixty-?nine)\\b" }
        responses: "Nice."

  - name: banana-bot
    identity:
      type: static
      bot_name: BananaBot
      avatar_url: https://static.wikia.nocookie.net/donkeykong/images/a/a6/Xananab.jpg
    triggers:
      - name: banana-trigger
        conditions: { contains_phrase: "banana" }
        responses:
          - "Always bring a :banana: to a party, banana's are good!"
          - "Don't drop the :banana:, they're a good source of potassium!"

  - name: sheesh-bot
    identity:
      type: static
      bot_name: Sheesh Bot
      avatar_url: https://i.imgflip.com/5fc2iz.png?a471000
    triggers:
      - name: sheesh-trigger
        conditions: { matches_regex: "\\bshe{2,}sh\\b" }
        responses: 'sh{random:2-20:e}sh 😤'

  - name: pickle-bot
    identity:
      type: static
      bot_name: GremlinBot
      avatar_url: https://i.imgur.com/D0czJFu.jpg
    triggers:
      - name: gremlin-trigger
        conditions: { contains_phrase: "gremlin" }
        responses: "Could you repeat that? I don't speak *gremlin*"

  - name: hold-bot
    identity:
      type: static
      bot_name: HoldBot
      avatar_url: https://media.tenor.com/kolcDjvQKGMAAAAe/joel-haver.png
    triggers:
      - name: hold-trigger
        conditions: { matches_regex: "^Hold\\.?$" }
        responses: "Hold."

  - name: attitude-bot
    identity:
      type: static
      bot_name: Xander Crews
      avatar_url: https://i.ytimg.com/vi/56PMgO3q2-A/sddefault.jpg
    triggers:
      - name: attitude-trigger
        conditions: { matches_regex: "(you|I|they|we) can'?t" }
        responses: "Well, not with *THAT* attitude!!!"

  - name: baby-bot
    identity:
      type: static
      bot_name: BabyBot
      avatar_url: https://i.redd.it/qc9qus78dc581.jpg
    triggers:
      - name: baby-trigger
        conditions: { contains_phrase: "baby" }
        responses: "https://media.tenor.com/NpnXNhWqKcwAAAAC/metroid-samus-aran.gif"

  - name: chaos-bot
    identity:
      type: static
      bot_name: ChaosBot
      avatar_url: https://preview.redd.it/md0lzbvuc3571.png
    triggers:
      - name: chaos-trigger
        conditions: { contains_phrase: "chaos" }
        responses: "All I know is...I'm here to kill Chaos"

  - name: gundam-bot
    identity:
      type: static
      bot_name: GundamBot
      avatar_url: https://a1.cdn.japantravel.com/photo/41317-179698/1440x960!/tokyo-unicorn-gundam-statue-in-odaiba-179698.jpg
    triggers:
      - name: gundam-trigger
        conditions: { matches_regex: "\\bg(u|a)ndam\\b" }
        responses: 'That''s the famous Unicorn Robot, "Gandum". There, I said it.'

  - name: interrupt-bot
    identity:
      type: static
      bot_name: Interrupt Bot
      avatar_url: https://i.imgur.com/YPFGEzM.png
    triggers:
      - name: interrupt-trigger
        conditions:
          all_of:
            - always: true
            - with_chance: 1
        responses:
          - "{start}-- Oh, sorry... go ahead"
          - "{start}-- Ah! sorry, I didn't mean to interrupt"
          - "{start}-- Wait, I-- nevermind, you were saying?"
          - "{start}-- Oh! Sorry, please continue"
          - "{start}-- Oops, didn't mean to cut you off"

  - name: venn-bot
    identity:
      type: mimic
      as_member: "151120340343455744"
    triggers:
      - name: venn-cringe-trigger
        conditions: { contains_phrase: "cringe" }
        responses:
          - 'Sorry, but that was über cringe...'
          - 'Geez, that was hella cringe...'

  - name: check-bot
    identity:
      type: static
      bot_name: CheckBot
      avatar_url: https://m.media-amazon.com/images/I/21Unzn9U8sL._AC_.jpg
    triggers:
      - name: check-trigger
        conditions: { matches_regex: "\\b(check)\\b" }
        responses:
          - "I believe you meant 'czech'!"
      - name: czech-trigger
        conditions: { matches_regex: "\\b(czech)\\b" }
        responses:
          - "I believe you meant 'check'!"

  - name: chad-bot
    identity:
      type: mimic
      as_member: "85184539906809856"
    triggers:
      - name: chad-random-trigger
        conditions:
          all_of:
            - always: true
            - with_chance: 1
        responses: "What is bro *yappin* about?..."

  - name: ezio-bot
    identity:
      type: static
      bot_name: Ezio Auditore Da Firenze
      avatar_url: https://www.creativeuncut.com/gallery-12/art/ac2-ezio5.jpg
    triggers:
      - name: ezio-trigger
        conditions: { matches_regex: "\\b(ezio|assassin)\\b" }
        responses:
          - "Remember, Nothing is true; Everything is permitted."

  - name: homonym-bot
    identity:
      type: static
      bot_name: Gerald
      avatar_url: https://i.imgur.com/zh1Jd8c.jpeg
    triggers:
      - name: their-there-trigger
        conditions:
          all_of:
            - matches_regex: "\\b(their|there|they're)\\b"
            - with_chance: 15
        responses: 'You mean "there"*'
      - name: your-youre-trigger
        conditions:
          all_of:
            - matches_regex: "\\b(your|you're)\\b"
            - with_chance: 15
        responses: 'You mean "you''re"*'
      - name: to-too-two-trigger
        conditions:
          all_of:
            - matches_regex: "\\b(to|too|two)\\b"
            - with_chance: 15
        responses: 'You mean "too"*'
"#;

#[test]
fn production_yaml_parses_all_19_bots() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("production YAML must parse");
    assert_eq!(bots.len(), 19);
}

#[test]
fn production_yaml_bot_names_are_correct() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let names: Vec<&str> = bots.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"botbot"));
    assert!(names.contains(&"guy-bot"));
    assert!(names.contains(&"clanker-bot"));
    assert!(names.contains(&"spider-bot"));
    assert!(names.contains(&"nice-bot"));
    assert!(names.contains(&"banana-bot"));
    assert!(names.contains(&"sheesh-bot"));
    assert!(names.contains(&"pickle-bot"));
    assert!(names.contains(&"hold-bot"));
    assert!(names.contains(&"attitude-bot"));
    assert!(names.contains(&"baby-bot"));
    assert!(names.contains(&"chaos-bot"));
    assert!(names.contains(&"gundam-bot"));
    assert!(names.contains(&"interrupt-bot"));
    assert!(names.contains(&"venn-bot"));
    assert!(names.contains(&"check-bot"));
    assert!(names.contains(&"chad-bot"));
    assert!(names.contains(&"ezio-bot"));
    assert!(names.contains(&"homonym-bot"));
}

// ---------------------------------------------------------------------------
// Identity types
// ---------------------------------------------------------------------------

#[test]
fn static_identity_parses_name_and_avatar() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity:
      type: static
      bot_name: CoolBot
      avatar_url: https://example.com/avatar.png
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert_eq!(
        bot.identity,
        IdentityConfig::Static {
            bot_name: "CoolBot".into(),
            avatar_url: "https://example.com/avatar.png".into(),
        }
    );
}

#[test]
fn mimic_identity_parses_as_member_string() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity:
      type: mimic
      as_member: "113035990725066752"
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert_eq!(
        bot.identity,
        IdentityConfig::Mimic {
            as_member: "113035990725066752".into()
        }
    );
}

#[test]
fn random_identity_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity:
      type: random
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert_eq!(bot.identity, IdentityConfig::Random);
}

// ---------------------------------------------------------------------------
// Condition node — leaf types
// ---------------------------------------------------------------------------

#[test]
fn condition_contains_phrase_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { contains_phrase: "banana" }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::ContainsPhrase("banana".into())
    );
}

#[test]
fn condition_contains_word_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { contains_word: "blue" }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::ContainsWord("blue".into())
    );
}

#[test]
fn condition_matches_regex_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { matches_regex: "\\bshe{2,}sh\\b" }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\bshe{2,}sh\b".into())
    );
}

#[test]
fn condition_matches_pattern_alias_parses() {
    // JS used both "matches_regex" and "matches_pattern" — both must work
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { matches_pattern: "\\btest\\b" }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\btest\b".into())
    );
}

#[test]
fn condition_from_user_parses_quoted_string() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { from_user: "113035990725066752" }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::FromUser(Snowflake("113035990725066752".into()))
    );
}

#[test]
fn condition_from_user_parses_bare_integer() {
    // Production bots.yml has: from_user: 113035990725066752 (unquoted)
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { from_user: 113035990725066752 }
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::FromUser(Snowflake("113035990725066752".into()))
    );
}

#[test]
fn condition_with_chance_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { with_chance: 15 }
"#,
    ));
    assert_eq!(bot.triggers[0].conditions, ConditionNode::WithChance(15));
}

#[test]
fn condition_always_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert_eq!(bot.triggers[0].conditions, ConditionNode::Always(true));
}

// ---------------------------------------------------------------------------
// Condition node — compound types
// ---------------------------------------------------------------------------

#[test]
fn condition_all_of_parses_two_children() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions:
          all_of:
            - contains_phrase: "banana"
            - with_chance: 25
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::ContainsPhrase("banana".into()),
            ConditionNode::WithChance(25),
        ])
    );
}

#[test]
fn condition_any_of_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions:
          any_of:
            - contains_word: "hello"
            - contains_word: "hi"
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AnyOf(vec![
            ConditionNode::ContainsWord("hello".into()),
            ConditionNode::ContainsWord("hi".into()),
        ])
    );
}

#[test]
fn condition_none_of_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions:
          none_of:
            - from_user: "99999"
            - contains_phrase: "spam"
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::NoneOf(vec![
            ConditionNode::FromUser(Snowflake("99999".into())),
            ConditionNode::ContainsPhrase("spam".into()),
        ])
    );
}

#[test]
fn condition_nested_compound_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions:
          all_of:
            - any_of:
                - contains_phrase: "hello"
                - contains_phrase: "hi"
            - with_chance: 50
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::AnyOf(vec![
                ConditionNode::ContainsPhrase("hello".into()),
                ConditionNode::ContainsPhrase("hi".into()),
            ]),
            ConditionNode::WithChance(50),
        ])
    );
}

// ---------------------------------------------------------------------------
// Backward compat: always + with_chance inside all_of (production idiom)
// ---------------------------------------------------------------------------

#[test]
fn backward_compat_always_with_chance_in_all_of_parses() {
    // Production pattern: all_of: [always: true, with_chance: N]
    // The `always: true` is redundant but must still parse correctly.
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions:
          all_of:
            - always: true
            - with_chance: 1
"#,
    ));
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::Always(true),
            ConditionNode::WithChance(1),
        ])
    );
}

// ---------------------------------------------------------------------------
// Response pools
// ---------------------------------------------------------------------------

#[test]
fn response_pool_single_string_parses_as_vec() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
        responses: "Nice."
"#,
    ));
    assert_eq!(bot.triggers[0].responses, vec!["Nice."]);
}

#[test]
fn response_pool_array_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
        responses:
          - "Hello!"
          - "Hi there!"
"#,
    ));
    assert_eq!(bot.triggers[0].responses, vec!["Hello!", "Hi there!"]);
}

#[test]
fn bot_level_response_pool_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    responses:
      - "Response one"
      - "Response two"
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert_eq!(bot.responses, vec!["Response one", "Response two"]);
}

#[test]
fn bot_level_response_pool_absent_defaults_to_empty() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert!(bot.responses.is_empty());
}

#[test]
fn trigger_response_pool_absent_defaults_to_empty() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert!(bot.triggers[0].responses.is_empty());
}

// ---------------------------------------------------------------------------
// Defaults: ignore_bots / ignore_humans
// ---------------------------------------------------------------------------

#[test]
fn ignore_bots_defaults_to_true() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert!(bot.ignore_bots, "ignore_bots should default to true");
}

#[test]
fn ignore_humans_defaults_to_false() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert!(!bot.ignore_humans, "ignore_humans should default to false");
}

#[test]
fn ignore_bots_false_and_ignore_humans_true_parses() {
    // botbot: responds only to bots, ignores humans
    let bot = parse_one(&wrap(
        r#"
  - name: botbot
    identity:
      type: static
      bot_name: BotBot
      avatar_url: https://example.com/avatar.png
    ignore_bots: false
    ignore_humans: true
    triggers:
      - conditions: { always: true }
        responses: Hello fellow bot!
"#,
    ));
    assert!(!bot.ignore_bots);
    assert!(bot.ignore_humans);
}

// ---------------------------------------------------------------------------
// Trigger names
// ---------------------------------------------------------------------------

#[test]
fn trigger_name_present_parses() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - name: my-trigger
        conditions: { always: true }
"#,
    ));
    assert_eq!(bot.triggers[0].name.as_deref(), Some("my-trigger"));
}

#[test]
fn trigger_name_absent_defaults_to_none() {
    let bot = parse_one(&wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { always: true }
"#,
    ));
    assert!(bot.triggers[0].name.is_none());
}

// ---------------------------------------------------------------------------
// Multiple triggers per bot
// ---------------------------------------------------------------------------

#[test]
fn multiple_triggers_parse_in_order() {
    let bot = parse_one(&wrap(
        r#"
  - name: check-bot
    identity:
      type: static
      bot_name: CheckBot
      avatar_url: https://example.com/check.png
    triggers:
      - name: check-trigger
        conditions: { matches_regex: "\\b(check)\\b" }
        responses:
          - "I believe you meant 'czech'!"
      - name: czech-trigger
        conditions: { matches_regex: "\\b(czech)\\b" }
        responses:
          - "I believe you meant 'check'!"
"#,
    ));
    assert_eq!(bot.triggers.len(), 2);
    assert_eq!(bot.triggers[0].name.as_deref(), Some("check-trigger"));
    assert_eq!(bot.triggers[1].name.as_deref(), Some("czech-trigger"));
}

// ---------------------------------------------------------------------------
// Per-bot production config verification
// ---------------------------------------------------------------------------

#[test]
fn botbot_config_is_correct() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "botbot").unwrap();

    assert!(!bot.ignore_bots, "botbot must respond to other bots");
    assert!(bot.ignore_humans, "botbot must ignore humans");
    assert_eq!(bot.triggers.len(), 1);
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::Always(true),
            ConditionNode::WithChance(1),
        ])
    );
    assert_eq!(bot.triggers[0].responses, vec!["Hello fellow bot!"]);
}

#[test]
fn guy_bot_config_is_correct() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "guy-bot").unwrap();

    assert_eq!(
        bot.identity,
        IdentityConfig::Mimic {
            as_member: "113035990725066752".into()
        }
    );
    assert!(
        !bot.responses.is_empty(),
        "guy-bot must have a bot-level response pool"
    );
    // The {random} template response must be in the pool
    assert!(
        bot.responses.iter().any(|r| r.contains("{random:")),
        "guy-bot pool must contain a {{random}} template"
    );
    assert_eq!(bot.triggers.len(), 2);
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::ContainsPhrase("guy".into())
    );
    assert_eq!(
        bot.triggers[1].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::FromUser(Snowflake("113035990725066752".into())),
            ConditionNode::WithChance(10),
        ])
    );
}

#[test]
fn sheesh_bot_config_is_correct() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "sheesh-bot").unwrap();

    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\bshe{2,}sh\b".into())
    );
    assert_eq!(bot.triggers[0].responses, vec!["sh{random:2-20:e}sh 😤"]);
}

#[test]
fn spider_bot_uses_negative_lookahead_regex() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "spider-bot").unwrap();

    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex("spider(?!-).*man".into())
    );
}

#[test]
fn nice_bot_has_single_string_response() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "nice-bot").unwrap();

    assert_eq!(bot.triggers[0].responses, vec!["Nice."]);
}

#[test]
fn interrupt_bot_config_is_correct() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "interrupt-bot").unwrap();

    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::Always(true),
            ConditionNode::WithChance(1),
        ])
    );
    assert_eq!(bot.triggers[0].responses.len(), 5);
    assert!(
        bot.triggers[0]
            .responses
            .iter()
            .all(|r| r.contains("{start}")),
        "all interrupt-bot responses must use the {{start}} template"
    );
}

#[test]
fn check_bot_has_two_triggers_in_correct_order() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "check-bot").unwrap();

    assert_eq!(bot.triggers.len(), 2);
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\b(check)\b".into())
    );
    assert_eq!(
        bot.triggers[1].conditions,
        ConditionNode::MatchesRegex(r"\b(czech)\b".into())
    );
}

#[test]
fn venn_bot_is_mimic_of_correct_user() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "venn-bot").unwrap();

    assert_eq!(
        bot.identity,
        IdentityConfig::Mimic {
            as_member: "151120340343455744".into()
        }
    );
}

#[test]
fn chad_bot_is_mimic_of_correct_user() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "chad-bot").unwrap();

    assert_eq!(
        bot.identity,
        IdentityConfig::Mimic {
            as_member: "85184539906809856".into()
        }
    );
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::AllOf(vec![
            ConditionNode::Always(true),
            ConditionNode::WithChance(1),
        ])
    );
}

#[test]
fn homonym_bot_has_three_triggers_each_with_chance_15() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "homonym-bot").unwrap();

    assert_eq!(bot.triggers.len(), 3);
    for trigger in &bot.triggers {
        if let ConditionNode::AllOf(children) = &trigger.conditions {
            assert!(
                children.contains(&ConditionNode::WithChance(15)),
                "each homonym-bot trigger must include with_chance: 15"
            );
        } else {
            panic!("homonym-bot trigger must be all_of");
        }
    }
}

#[test]
fn hold_bot_regex_anchored_to_full_message() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "hold-bot").unwrap();

    // Regex must be anchored: ^Hold\.?$ so "Hold on" doesn't trigger
    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"^Hold\.?$".into())
    );
}

#[test]
fn ezio_bot_matches_both_name_and_class() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "ezio-bot").unwrap();

    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\b(ezio|assassin)\b".into())
    );
}

#[test]
fn gundam_bot_matches_both_spellings() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "gundam-bot").unwrap();

    assert_eq!(
        bot.triggers[0].conditions,
        ConditionNode::MatchesRegex(r"\bg(u|a)ndam\b".into())
    );
}

#[test]
fn attitude_bot_bot_name_contains_space() {
    let bots = parse_bots(PRODUCTION_BOTS_YAML).expect("parse");
    let bot = bots.iter().find(|b| b.name == "attitude-bot").unwrap();

    if let IdentityConfig::Static { bot_name, .. } = &bot.identity {
        assert_eq!(bot_name, "Xander Crews");
    } else {
        panic!("attitude-bot must have a static identity");
    }
}

// ---------------------------------------------------------------------------
// Snowflake deserialization edge cases
// ---------------------------------------------------------------------------

#[test]
fn snowflake_from_bare_integer_matches_snowflake_from_quoted_string() {
    // Ensure both forms produce the same Snowflake value
    let yaml_int = wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { from_user: 113035990725066752 }
"#,
    );
    let yaml_str = wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { from_user: "113035990725066752" }
"#,
    );

    let bot_int = parse_one(&yaml_int);
    let bot_str = parse_one(&yaml_str);

    assert_eq!(
        bot_int.triggers[0].conditions,
        bot_str.triggers[0].conditions
    );
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn missing_triggers_field_causes_parse_error() {
    let yaml = wrap(
        r#"
  - name: test
    identity: { type: random }
"#,
    );
    assert!(
        parse_bots(&yaml).is_err(),
        "a bot without triggers must fail to parse"
    );
}

#[test]
fn missing_identity_field_causes_parse_error() {
    let yaml = wrap(
        r#"
  - name: test
    triggers:
      - conditions: { always: true }
"#,
    );
    assert!(
        parse_bots(&yaml).is_err(),
        "a bot without identity must fail to parse"
    );
}

#[test]
fn unknown_identity_type_causes_parse_error() {
    let yaml = wrap(
        r#"
  - name: test
    identity:
      type: morphic
      some_field: value
    triggers:
      - conditions: { always: true }
"#,
    );
    assert!(
        parse_bots(&yaml).is_err(),
        "an unknown identity type must fail to parse"
    );
}

#[test]
fn with_chance_over_100_still_parses_as_u8_wraps() {
    // Validation (clamping to 0–100) is enforced at strategy build time, not here.
    // Parsing itself should not reject it — the u8 will just overflow or saturate
    // depending on the value. Values <= 255 parse; > 255 cause serde error.
    let yaml = wrap(
        r#"
  - name: test
    identity: { type: random }
    triggers:
      - conditions: { with_chance: 100 }
"#,
    );
    assert!(parse_bots(&yaml).is_ok(), "with_chance: 100 must parse");
}

#[test]
fn empty_reply_bots_list_parses_to_empty_vec() {
    let yaml = "reply-bots: []";
    let bots = parse_bots(yaml).expect("empty list must parse");
    assert!(bots.is_empty());
}
