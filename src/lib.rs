#![allow(non_snake_case)]

use std::collections::{HashSet, VecDeque};

use leptos::prelude::*;
use leptos::*;
use rand::Rng;
use web_sys::{Attr, Event};
use web_time::Instant;

mod record;

use record::Record;

type SignalPair<T> = (ReadSignal<T>, WriteSignal<T>);
type Position = (u32, u32);
type Positions = HashSet<Position, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[component]
pub fn App() -> impl IntoView {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let columns = signal(
        local_storage
            .get_item("columns")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| 3),
    );
    let rows = signal(
        local_storage
            .get_item("rows")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| 3),
    );
    let active = signal(
        local_storage
            .get_item("active")
            .unwrap()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| 3u32),
    );

    let current: SignalPair<Positions> = signal(HashSet::with_capacity_and_hasher(
        (active.0.get_untracked() + 1) as usize,
        Default::default(),
    ));
    let history = local_storage
        .get_item("history")
        .unwrap()
        .map(|s| {
            s.split('\n')
                .filter_map(Record::from_str)
                .collect::<VecDeque<_>>()
        })
        .unwrap_or_else(VecDeque::new);
    if history.is_empty() {
        local_storage.delete("history").unwrap();
    }

    let history = signal(history);

    let current_record = signal(Record::new(
        0,
        0,
        0,
        rows.0.get_untracked(),
        columns.0.get_untracked(),
        active.0.get_untracked(),
    ));
    let score = move || current_record.0().score;

    let best_record = Memo::new(move |_| {
        let mut history_obj = history.0();
        history_obj.retain(|e: &Record| {
            e.rows == rows.0() && e.columns == columns.0() && e.active == active.0()
        });
        std::iter::once(current_record.0())
            .chain(history_obj.iter().copied())
            .max_by(|a, b| {
                use std::cmp::Ordering::*;
                match a.score.cmp(&b.score) {
                    Equal => b.millis.cmp(&a.millis),
                    otherwise => otherwise,
                }
            })
            .unwrap_or_else(|| {
                Record::new(
                    0,
                    0,
                    0,
                    rows.0.get_untracked(),
                    columns.0.get_untracked(),
                    active.0.get_untracked(),
                )
            })
    });

    let update_current = move || {
        let active = active.0().min(columns.0() * rows.0() - 1);
        let mut rng = rand::rng();

        current.1.update(|current| {
            current.clear();
            while current.len() < active as usize {
                let new = (
                    rng.random_range(0..rows.0()),
                    rng.random_range(0..columns.0()),
                );
                if current.contains(&new) {
                    continue;
                }

                current.insert(new);
            }
        });
    };

    let max_active = Memo::new(move |_| rows.0() * columns.0() - 1);
    let score_text = Memo::new(move |_| {
        format!(
            "Score: {} ({:.2}/s) / {} ({:.2}/s)",
            score(),
            (score() * 1000) as f64 / current_record.0().millis as f64,
            best_record().score,
            (best_record().score * 1000) as f64 / best_record().millis as f64
        )
    });

    view! {
        <div style="display: flex; justify-content: space-evenly;">
            <U32Input name="rows" label="Rows: " min=2 max=u32::MAX signal=rows current=current.1 onchange=update_current />
            <U32Input name="columns" label="Columns: " min=2 max=u32::MAX signal=columns current=current.1 onchange=update_current />
            <U32Input name="active" label="Active: " min=1 max=max_active signal=active current=current.1 onchange=update_current />
            <button on:click=move |_| {
                history.1.update(|history| {
                    history.clear();
                    local_storage.delete("history").unwrap();
                });
            }>"Clear History"</button>
        </div>

        <Game current={current} history={history} columns={columns.0} rows={rows.0} active={active.0} current_record={current_record} />

        <h3 style="text-align: center;">{score_text}</h3>
        <GameHistory history={history.0} />
    }
}

#[component]
fn U32Input<F>(
    name: &'static str,
    label: &'static str,
    min: u32,
    #[prop(into)] max: Signal<u32>,
    signal: SignalPair<u32>,
    current: WriteSignal<Positions>,
    onchange: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    view! {
        <span>
            <label for=name>{label}</label>
            <input
                name=name
                type="number"
                min=min
                max=max
                value=signal.0
                on:change=move |ev| {
                    signal.1(event_target_value(&ev).parse().unwrap_or_else(|_| signal.0()));
                    current.update(|current| current.clear());
                    local_storage.set(name, &signal.0().to_string()).unwrap();
                    onchange();
                }
            />
        </span>
    }
}

#[component]
fn GameHistory(history: ReadSignal<VecDeque<Record>>) -> impl IntoView {
    view! {
        <table class="GameHistory">
            <tr class="GameHistory">
                <th class="GameHistory">"Pos"</th>
                <th class="GameHistory">"Score"</th>
                <th class="GameHistory">"Score/s"</th>
                <th class="GameHistory">"Seconds"</th>
                <th class="GameHistory">"Size"</th>
                <th class="GameHistory">"Active"</th>
            </tr>

            <For
                each=history
                key=|record| record.position
                children=move |record| {
                    view! {
                        <tr class="GameHistory">
                            <td class="GameHistory">{record.position}</td>
                            <td class="GameHistory">{record.score}</td>
                            <td class="GameHistory">{format!("{:.2}", (record.score * 1000) as f64 / record.millis as f64)}</td>
                            <td class="GameHistory">{format!("{:.2}", record.millis as f64 / 1000f64)}</td>
                            <td class="GameHistory">{format!("{}×{}", record.rows, record.columns)}</td>
                            <td class="GameHistory">{format!("{}", record.active)}</td>
                        </tr>
                    }
                }
            />
        </table>
    }
}

#[component]
fn Game(
    current: SignalPair<Positions>,
    history: SignalPair<VecDeque<Record>>,
    columns: ReadSignal<u32>,
    rows: ReadSignal<u32>,
    active: ReadSignal<u32>,
    current_record: SignalPair<Record>,
) -> impl IntoView {
    let (current, set_current) = current;
    let (history, set_history) = history;
    let (current_record, set_current_record) = current_record;

    let (start, set_start) = signal(Instant::now());
    let (hovered, set_hovered) = signal(None);

    let active = Memo::new(move |_| active().min(rows() * columns() - 1));

    let mut rng = rand::rng();
    set_current.update(|current| {
        current.clear();
        while current.len() < active.get_untracked() as usize {
            let new = (
                rng.random_range(0..rows.get_untracked()),
                rng.random_range(0..columns.get_untracked()),
            );
            if current.contains(&new) {
                continue;
            }

            current.insert(new);
        }
    });

    let game_over = move || {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        if current_record().score > 1 {
            set_history.update(|history| {
                history.push_front(Record::new(
                    history.len() as u32 + 1,
                    current_record().score,
                    current_record().millis,
                    rows.get_untracked(),
                    columns.get_untracked(),
                    active.get_untracked(),
                ))
            });

            local_storage
                .set_item(
                    "history",
                    &history()
                        .iter()
                        .map(Record::to_string)
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .unwrap();
        }
        set_current_record.update(|record| record.score = 0);
    };

    let on_input = move |row, col| {
        if current().contains(&(row, col)) {
            let now = Instant::now();
            set_current.update(|current| {
                let mut rng = rand::rng();

                if current_record().score == 0 {
                    set_start(now);
                }

                set_current_record.update(|record| {
                    record.millis = (now - start()).as_millis();
                    record.score += 1;
                });

                let mut new = (rng.random_range(0..rows()), rng.random_range(0..columns()));
                while current.contains(&new) {
                    new = (rng.random_range(0..rows()), rng.random_range(0..columns()));
                }
                current.remove(&(row, col));
                current.insert(new);
            });
            return;
        }

        game_over();
    };

    let on_trigger = move |ev: Event| {
        if let Some((row, col)) = hovered() {
            on_input(row, col);
            ev.prevent_default();
        } else {
            game_over()
        }
    };

    window_event_listener(ev::keydown, move |ev| on_trigger(ev.into()));
    window_event_listener(ev::touchstart, move |ev| on_trigger(ev.into()));
    window_event_listener(ev::mousedown, move |ev| on_trigger(ev.into()));

    window_event_listener(ev::mouseover, move |ev| {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_hovered(None);
            return;
        };

        let Ok(element) = target.dyn_into::<web_sys::Element>() else {
            return;
        };

        if element.class_list().contains("cell") {
            let attrs = element.attributes();
            let row = attrs
                .get_named_item("data-row")
                .map(|a| Attr::value(&a).parse());
            let col = attrs
                .get_named_item("data-col")
                .map(|a| Attr::value(&a).parse());

            match (row, col) {
                (Some(Ok(row)), Some(Ok(col))) => set_hovered(Some((row, col))),
                _ => set_hovered(None),
            }
        } else {
            set_hovered(None);
        }
    });

    view! {
        <div class="Game container">
            <div class="Game grid" style=("--columns", move || columns().to_string()) style=("--rows", move || rows().to_string())>
                <For
                    each=move || 0..rows()
                    key=|&idx| idx
                    children=move |row| {
                        view! {
                            <div class="Game">
                                <For
                                    each=move || 0..columns()
                                    key=|idx| *idx
                                    children=move |col| {
                                        view! {
                                            <div
                                                class="Game cell"
                                                data-row=row
                                                data-col=col
                                                class:active=move || current().contains(&(row, col))
                                            />
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
