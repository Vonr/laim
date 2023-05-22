use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use rand::Rng;
use serde::*;
use stylers::*;
use web_time::Instant;

type SignalPair<T> = (ReadSignal<T>, WriteSignal<T>);
type Position = (usize, usize);

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Record(u64, u64, u128, usize, usize);

#[allow(dead_code)]
impl Record {
    #[inline]
    pub const fn new(position: u64, score: u64, millis: u128, rows: usize, columns: usize) -> Self {
        Self(position, score, millis, rows, columns)
    }

    #[inline]
    pub const fn position(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn set_position(&mut self, value: u64) {
        self.0 = value;
    }

    #[inline]
    pub const fn score(&self) -> u64 {
        self.1
    }

    #[inline]
    pub fn set_score(&mut self, value: u64) {
        self.1 = value;
    }

    #[inline]
    pub const fn millis(&self) -> u128 {
        self.2
    }

    #[inline]
    pub fn set_millis(&mut self, value: u128) {
        self.2 = value;
    }

    #[inline]
    pub const fn rows(&self) -> usize {
        self.3
    }

    #[inline]
    pub fn set_rows(&mut self, value: usize) {
        self.3 = value;
    }

    #[inline]
    pub const fn columns(&self) -> usize {
        self.4
    }

    #[inline]
    pub fn set_columns(&mut self, value: usize) {
        self.4 = value;
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let columns = create_signal(cx, LocalStorage::get("columns").unwrap_or(3));
    let rows = create_signal(cx, LocalStorage::get("rows").unwrap_or(3));
    let active = create_signal(cx, LocalStorage::get("active").unwrap_or(3));

    let current = create_signal(cx, Vec::with_capacity(active.0() + 1));
    let history = create_signal(
        cx,
        LocalStorage::get("history").unwrap_or_else(|_| Vec::new()),
    );

    let current_record = create_signal(cx, Record::new(0, 0, 0, rows.0(), columns.0()));
    let score = move || current_record.0().score();

    let history_best = move || {
        let mut history_obj = history.0();
        history_obj.retain(|e: &Record| e.rows() == rows.0() && e.columns() == columns.0());
        history_obj.sort_by(|a: &Record, b: &Record| {
            use std::cmp::Ordering::*;
            match a.score().cmp(&b.score()) {
                Equal => b.millis().cmp(&a.millis()),
                otherwise => otherwise,
            }
        });
        history_obj
            .last()
            .copied()
            .unwrap_or_else(|| Record::new(0, 0, 0, rows.0(), columns.0()))
    };

    let best_record = create_signal(cx, history_best());

    let update_current = move || {
        let active = active.0().min(columns.0() * rows.0() - 1);
        let mut rng = rand::thread_rng();
        current.1.update(|current| {
            current.clear();
            while current.len() < active {
                let new = (rng.gen_range(0..rows.0()), rng.gen_range(0..columns.0()));
                if current.contains(&new) {
                    continue;
                }

                current.push(new);
            }
        });
    };

    view! { cx,
        <div style="display: flex; justify-content: space-evenly;">
            <UsizeInput name="rows" label="Rows: " min=2 max=|| usize::MAX signal=rows current=current.1 onchange=update_current />
            <UsizeInput name="columns" label="Columns: " min=2 max=|| usize::MAX signal=columns current=current.1 onchange=update_current />
            <UsizeInput name="active" label="Active: " min=1 max=move || rows.0() * columns.0() - 1 signal=active current=current.1 onchange=update_current />
            <button on:click=move |_| {
                best_record.1.update(|record| {
                    record.set_millis(0);
                    record.set_millis(0);
                });
                history.1.update(|history| {
                    history.clear();
                    LocalStorage::delete("history");
                });
            }>"Clear History"</button>
        </div>

        <Game current={current} history={history} columns={columns.0} rows={rows.0} active={active.0} current_record={current_record} best_record={best_record} />

        <h3 style="text-align: center;">{move || format!("Score: {} ({:.2}/s) / {} ({:.2}/s)",
            score(),
            (score() * 1000) as f64 / current_record.0().millis() as f64,
            history_best().score(), (history_best().score() * 1000) as f64 / history_best().millis() as f64
        )}</h3>
        <GameHistory history={history.0} />
    }
}

#[component]
fn UsizeInput<F, G>(
    cx: Scope,
    name: &'static str,
    label: &'static str,
    min: usize,
    max: F,
    signal: SignalPair<usize>,
    current: WriteSignal<Vec<Position>>,
    onchange: G,
) -> impl IntoView
where
    F: Fn() -> usize + 'static,
    G: Fn() + 'static,
{
    view! { cx,
        <span>
            <label for=name>{label}</label>
            <input
                name=name
                type="number"
                min=min
                max=max
                value={signal.0}
                on:change=move |ev| {
                    signal.1(event_target_value(&ev).parse().unwrap_or_else(|_| signal.0()));
                    current.update(|current| current.clear());
                    let _ = LocalStorage::set(name, signal.0());
                    onchange();
                }
            />
        </span>
    }
}

#[component]
fn GameHistory(cx: Scope, history: ReadSignal<Vec<Record>>) -> impl IntoView {
    let style = style! {"GameHistory",
        table {
            max-width: 90%;
            border-collapse: collapse;
            border: 1px solid black;
            margin: auto;
        }

        th, td {
            text-align: center;
            border: 1px solid black;
            padding-left: 1rem;
            padding-right: 1rem;
            text-align: left;
        }

        tr>:nth-child(1) {
            width: 8rem;
        }
    };

    view! { cx, class = style,
        <table>
            <tr>
                <th>"Position"</th>
                <th>"Score"</th>
                <th>"Score/s"</th>
                <th>"Seconds"</th>
                <th>"Size"</th>
            </tr>

            <For
                each=history
                key=|record| record.0
                view=move |cx, record| {
                    view! { cx, class = style,
                        <tr>
                            <td>{record.position()}</td>
                            <td>{record.score()}</td>
                            <td>{format!("{:.2}", (record.score() * 1000) as f64 / record.millis() as f64)}</td>
                            <td>{format!("{:.2}", record.millis() as f64 / 1000f64)}</td>
                            <td>{format!("{}x{}", record.rows(), record.columns())}</td>
                        </tr>
                    }
                }
            />
        </table>
    }
}

#[component]
fn Game(
    cx: Scope,
    current: SignalPair<Vec<Position>>,
    history: SignalPair<Vec<Record>>,
    columns: ReadSignal<usize>,
    rows: ReadSignal<usize>,
    active: ReadSignal<usize>,
    current_record: SignalPair<Record>,
    best_record: SignalPair<Record>,
) -> impl IntoView {
    let (current, set_current) = current;
    let (history, set_history) = history;
    let (current_record, set_current_record) = current_record;
    let (best_record, set_best_record) = best_record;

    let (start, set_start) = create_signal(cx, Instant::now());
    let (hovered, set_hovered) = create_signal(cx, None);

    let active = move || active().min(rows() * columns() - 1);

    let mut rng = rand::thread_rng();
    set_current.update(|current| {
        current.clear();
        let active = active().min(columns() * rows() - 1);
        while current.len() < active {
            let new = (rng.gen_range(0..rows()), rng.gen_range(0..columns()));
            if current.contains(&new) {
                continue;
            }

            current.push(new);
        }
    });

    let game_over = move || {
        let curr = current_record();
        if curr.score() > 1 {
            set_history.update(|history| {
                history.insert(
                    0,
                    Record::new(
                        history.len() as u64 + 1,
                        curr.score(),
                        curr.millis(),
                        rows(),
                        columns(),
                    ),
                )
            });

            let _ = LocalStorage::set("history", history());
        }
        set_current_record.update(|record| record.1 = 0);
    };

    let on_input = move |row, col| {
        if let Some(idx) = current().iter().position(|&pos| pos == (row, col)) {
            let mut rng = rand::thread_rng();
            set_current.update(|current| {
                let active = active().min(columns() * rows() - 1);
                while current.len() < active + 1 {
                    let new = (rng.gen_range(0..rows()), rng.gen_range(0..columns()));
                    if current.contains(&new) {
                        continue;
                    }

                    current.push(new);
                }
                current.remove(idx);

                if current_record().score() == 0 {
                    set_start(Instant::now());
                }

                set_current_record.update(|record| {
                    record.set_millis((Instant::now() - start()).as_millis());
                    record.set_score(record.score() + 1)
                });

                let score = move || current_record().score();
                let best_score = move || best_record().score();
                let millis = move || current_record().millis();
                let best_millis = move || best_record().millis();
                if score() > best_score() || (score() == best_score() && millis() < best_millis()) {
                    set_best_record.update(|record| {
                        record.set_score(score());
                        record.set_millis(millis());
                    });
                }
            });
            return;
        }

        game_over();
    };

    window_event_listener(ev::keypress, move |ev| {
        if let Some((row, col)) = hovered() {
            on_input(row, col);
            ev.prevent_default();
        } else {
            game_over()
        }
    });

    window_event_listener(ev::mousedown, move |_| {
        if hovered().is_none() {
            game_over();
        }
    });

    window_event_listener(ev::mouseover, move |ev| {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_hovered(None);
            return;
        };

        let element = target.unchecked_into::<web_sys::Element>();

        if element.class_list().contains("cell") {
            let attrs = element.attributes();
            let row = attrs.get_named_item("data-row");
            let col = attrs.get_named_item("data-col");
            match (row, col) {
                (Some(row), Some(col)) => {
                    let row = row.value().parse();
                    let col = col.value().parse();
                    match (row, col) {
                        (Ok(row), Ok(col)) => set_hovered(Some((row, col))),
                        _ => set_hovered(None),
                    }
                }
                _ => set_hovered(None),
            }
        } else {
            set_hovered(None);
        }
    });

    let style = style! {"Game",
        .container {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 90vh;
            margin-top: 1rem;
        }

        .grid:deep() {
            display: grid;
            grid-template-columns: repeat(var(--columns), 0);
            grid-template-rows: repeat(var(--rows), 0);
            width: 50%;
            height: 100%;
            border: 1px solid black;
            box-sizing: border-box;
        }

        .cell:deep() {
            display: inline-block;
            border: 1px solid black;
            box-sizing: border-box;
            width: calc(100% / var(--columns));
            height: 100%;
        }

        .active:deep() {
            background-color: black;
            border: 0.5px solid grey;
            -webkit-animation-name: fadeIn;
            animation-name: fadeIn;
            -webkit-animation-duration: 0.15s;
            animation-duration: 0.15s;
        }

         @-webkit-keyframes fadeIn {
            0% {opacity: 0;}
            100% {opacity: 1;}
         }

         @keyframes fadeIn {
            0% {opacity: 0;}
            100% {opacity: 1;}
         }
    };

    view! { cx, class = style,
        <div class="container">
            <div class="grid">
                <For
                    each=move || {0..rows.get()}
                    key=|&idx| idx
                    view=move |cx, row| {
                        view! { cx,
                        <div>
                            <For
                                each=move || {0..columns.get()}
                                key=|idx| *idx
                                view=move |cx, col| {
                                    view! { cx,
                                        <div
                                            class="cell"
                                            data-row=row
                                            data-col=col
                                            style=("--columns", columns)
                                            style=("--rows", rows)
                                            class:active=move || current().contains(&(row, col))
                                            on:mousedown=move |_| on_input(row, col)
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
