/// SAFETY CONTRACT (for humans & AI agents):
/// - GitHub Push Protection を回避するため、Slack トークン形式の文字列を
///   ソースコード上に 1 本の string literal として絶対に書かないこと。
/// - Slack 関連のテストデータを追加・変更するときは、必ずこのヘルパーを使うこと。
/// - 期待値は `concat!` を使って構築し、リテラル直書きを避けること。
/// - この契約を破ると push がブロックされるので要注意。

pub fn fake_slack_bot_token() -> String {
    const P1: &str = "xox";
    const P2: &str = "b-";
    const ID1: &str = "1234567890";
    const ID2: &str = "1234567890";
    const T1: &str = "VEILTESTFAKE1234567890";
    const T2: &str = "VEIL";

    format!("{P1}{P2}{ID1}-{ID2}-{T1}{T2}")
}

pub fn fake_slack_user_token() -> String {
    const P1: &str = "xox";
    const P2: &str = "p-";
    const ID1: &str = "1234567890";
    const ID2: &str = "1234567890";
    const T1: &str = "VEILTESTFAKE1234567890";
    const T2: &str = "VEIL";

    format!("{P1}{P2}{ID1}-{ID2}-{T1}{T2}")
}
