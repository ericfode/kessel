//! Closed contracts for the ordinary native deck-search family.
//!
//! This module deliberately does not make deck search executable. The native
//! remaining-card handler shuffles the unchosen cards before bottoming them,
//! including when the player takes nothing. Whole-deck searches always shuffle.
//! Neither operation can be replaced by a player choice or a stable sort in the
//! deterministic prover. Keep this typed boundary until chance/RNG snapshots and
//! continuation/public-display state have an explicit execution contract.
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Constant<T> {
    Const { value: T },
}
impl<T: Copy> Constant<T> {
    fn value(&self) -> T {
        match self {
            Self::Const { value } => *value,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Player {
    OwnPlayer,
    EnemyPlayer,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TargetMode {
    UpTo,
    Single,
    Unlimited,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Remaining {
    BottomInRandomOrderUnlessWholeDeck,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Stage {
    Remaining,
    Selected,
    Shuffle,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SelectedContext {
    SelectedPromptCards,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SelectedTarget {
    SearchSelected,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PromptedPlayer {
    Opponent,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Evaluation {
    SearchCardFilter,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Predicate {
    Const {
        value: bool,
    },
    SystemValue {
        evaluation: Evaluation,
        value: CardPredicate,
    },
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum CardPredicate {
    Query {
        name: Query,
        receiver: CardRef,
        args: Vec<Constant<String>>,
    },
    Binary {
        op: BooleanOp,
        left: Box<CardPredicate>,
        right: Box<CardPredicate>,
    },
    Unary {
        op: Not,
        value: Box<CardPredicate>,
    },
}
#[derive(Debug, Deserialize)]
enum BooleanOp {
    #[serde(rename = "&&")]
    And,
    #[serde(rename = "||")]
    Or,
}
#[derive(Debug, Deserialize)]
enum Not {
    #[serde(rename = "!")]
    Not,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Query {
    IsUnit,
    IsUpgrade,
    IsEvent,
    HasSomeTrait,
    HasSomeAspect,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum CardRef {
    Ref { path: Vec<String> },
}
impl CardPredicate {
    fn validate(&self, depth: usize) -> Result<(), String> {
        if depth > 8 {
            return Err("deck-search predicate exceeds bounded depth".into());
        }
        match self {
            Self::Query {
                name,
                receiver: CardRef::Ref { path },
                args,
            } => {
                if path != &["card"] {
                    return Err("deck-search candidate must be the physical search card".into());
                }
                let typed = matches!(name, Query::IsUnit | Query::IsUpgrade | Query::IsEvent);
                if typed && !args.is_empty()
                    || !typed
                        && (args.len() != 1
                            || !matches!(&args[0], Constant::Const { value } if !value.is_empty() && value.len() <= 64))
                {
                    return Err("deck-search query needs its exact literal argument shape".into());
                }
            }
            Self::Binary { op, left, right } => {
                let _ = op;
                left.validate(depth + 1)?;
                right.validate(depth + 1)?;
            }
            Self::Unary { op, value } => {
                let _ = op;
                value.validate(depth + 1)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum SelectedEffect {
    RevealAndDraw {
        target: SelectedTarget,
        #[serde(rename = "useDisplayPrompt")]
        use_display_prompt: Option<bool>,
        #[serde(rename = "promptedPlayer")]
        prompted_player: Option<PromptedPlayer>,
    },
    DrawSpecificCard {
        target: SelectedTarget,
        switch: bool,
        shuffle: bool,
        #[serde(rename = "changePlayer")]
        change_player: bool,
    },
    DiscardSpecificCard {
        target: SelectedTarget,
    },
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Search {
    DeckSearch {
        target: Player,
        #[serde(rename = "targetMode")]
        target_mode: TargetMode,
        #[serde(rename = "searchWholeDeck")]
        search_whole_deck: Constant<bool>,
        #[serde(rename = "searchCount")]
        search_count: Option<Constant<u8>>,
        #[serde(rename = "selectCount")]
        select_count: Constant<u8>,
        #[serde(rename = "shuffleWhenDone")]
        shuffle_when_done: Constant<bool>,
        #[serde(rename = "canChooseFewer")]
        can_choose_fewer: Option<Constant<bool>>,
        #[serde(rename = "cardCondition")]
        card_condition: Predicate,
        #[serde(rename = "multiSelectCondition")]
        multi_select_condition: Constant<bool>,
        #[serde(rename = "remainingCardsDefault")]
        remaining_cards_default: Remaining,
        #[serde(rename = "wholeDeckAlwaysShuffles")]
        whole_deck_always_shuffles: bool,
        #[serde(rename = "completionOrder")]
        completion_order: [Stage; 3],
        #[serde(rename = "selectedContextField")]
        selected_context_field: SelectedContext,
        #[serde(rename = "selectedCardsImmediateEffect")]
        selected_cards_immediate_effect: SelectedEffect,
        #[serde(rename = "activePromptTitleParts")]
        active_prompt_title_parts: Option<Vec<Constant<String>>>,
    },
}
impl Search {
    fn blocker(&self) -> Result<&'static str, String> {
        let Self::DeckSearch {
            target,
            target_mode,
            search_whole_deck,
            search_count,
            select_count,
            shuffle_when_done,
            can_choose_fewer,
            card_condition,
            multi_select_condition,
            remaining_cards_default,
            whole_deck_always_shuffles,
            completion_order,
            selected_context_field,
            selected_cards_immediate_effect,
            active_prompt_title_parts,
        } = self;
        let _ = (target, remaining_cards_default, selected_context_field);
        if !matches!(target_mode, TargetMode::UpTo | TargetMode::Single)
            || !(1..=5).contains(&select_count.value())
        {
            return Err(
                "deck search supports only bounded single/up-to selection contracts".into(),
            );
        }
        if !whole_deck_always_shuffles
            || !matches!(
                completion_order,
                [Stage::Remaining, Stage::Selected, Stage::Shuffle]
            )
            || !multi_select_condition.value()
        {
            return Err(
                "deck search requires exact native completion order and selection defaults".into(),
            );
        }
        // Native SearchDeckSystem uses `properties.canChooseFewer || true`:
        // even the explicit false source value still permits taking nothing.
        if let Some(value) = can_choose_fewer {
            let _ = value.value();
        }
        if active_prompt_title_parts.as_ref().is_some_and(|parts| {
            parts.len() > 16
                || parts
                    .iter()
                    .any(|p| matches!(p, Constant::Const { value } if value.len() > 256))
        }) {
            return Err("deck-search prompt exceeds bounded literal text".into());
        }
        match card_condition {
            Predicate::Const { value: true } => {}
            Predicate::Const { value: false } => {
                return Err(
                    "constant-false deck-search filter is not a reviewed source contract".into(),
                )
            }
            Predicate::SystemValue { evaluation, value } => {
                let _ = evaluation;
                value.validate(0)?;
            }
        }
        match selected_cards_immediate_effect {
            SelectedEffect::DrawSpecificCard {
                target,
                switch,
                shuffle,
                change_player,
            } => {
                let _ = target;
                if *switch || *shuffle || *change_player {
                    return Err(
                        "deck search cannot erase selected-card switch/shuffle/control semantics"
                            .into(),
                    );
                }
            }
            SelectedEffect::RevealAndDraw {
                target,
                use_display_prompt,
                prompted_player,
            } => {
                let _ = (target, use_display_prompt, prompted_player);
            }
            SelectedEffect::DiscardSpecificCard { target } => {
                let _ = target;
            }
        }
        if search_whole_deck.value() {
            if search_count.is_some() {
                return Err("whole-deck search must omit searchCount".into());
            }
            return Ok("whole-deck search always shuffles; exact RNG snapshot or chance semantics are unimplemented");
        }
        let count = search_count
            .as_ref()
            .ok_or("top-N deck search requires literal searchCount")?
            .value();
        if !(1..=64).contains(&count) {
            return Err("deck-search count must fit bounded deck inventory".into());
        }
        if shuffle_when_done.value() {
            return Ok("deck search requests a final shuffle; exact RNG snapshot or chance semantics are unimplemented");
        }
        if count > 1 {
            return Ok("deck search bottoms unchosen cards in random order, including take-nothing; exact RNG snapshot or chance semantics are unimplemented");
        }
        Ok("deck-search display/selection/completion continuations are unimplemented")
    }
}

/// Find deck-search effects without admitting any part of their containing card.
/// Known forms get a specific missing execution contract. Unknown fields,
/// dynamic/private callbacks and changed native defaults stay explicit failures.
pub(crate) fn reject(value: &Value) -> Result<(), String> {
    fn walk(value: &Value, depth: usize) -> Result<(), String> {
        if depth > 64 {
            return Err("deck-search inspection exceeds bounded IR depth".into());
        }
        if value["kind"] == "deckSearch" {
            let search: Search = serde_json::from_value(value.clone())
                .map_err(|e| format!("unimplemented deck-search contract: {e}"))?;
            return Err(format!(
                "unimplemented deck-search contract: {}",
                search.blocker()?
            ));
        }
        match value {
            Value::Object(object) => {
                for child in object.values() {
                    walk(child, depth + 1)?;
                }
            }
            Value::Array(array) => {
                for child in array {
                    walk(child, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    walk(value, 0)
}

#[cfg(test)]
#[path = "deck_search_tests.rs"]
mod tests;
