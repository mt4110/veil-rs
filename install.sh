#!/bin/sh
# Offline installer for an extracted, checksum-verified Veil CLI archive.
set -eu
umask 077

lang=${VEIL_LANG-}
prefix=${HOME:+$HOME/.local}
activate=false
help=false
while [ "$#" -gt 0 ]; do
    case $1 in
        --prefix|--lang)
            [ "$#" -ge 2 ] || { printf '%s\n' 'Missing option value / オプション値が必要です' >&2; exit 2; }
            case $1 in --prefix) prefix=$2 ;; --lang) lang=$2 ;; esac
            shift 2 ;;
        --activate) activate=true; shift ;;
        --help|-h) help=true; shift ;;
        *) printf '%s\n' 'Unknown option / 不明なオプションです; use --help' >&2; exit 2 ;;
    esac
done
if [ -z "$lang" ]; then
    locale=${LC_ALL:-${LC_MESSAGES:-${LANG:-C}}}
    case $locale in ja|ja[_.@-]*) lang=ja ;; *) lang=en ;; esac
fi
case $lang in ja|en) ;; *) printf '%s\n' 'Use --lang ja or en / --lang ja または en を指定してください' >&2; exit 2 ;; esac
say() { if [ "$lang" = ja ]; then printf '%s\n' "$2"; else printf '%s\n' "$1"; fi; }
fail() { say "$1" "$2" >&2; exit 2; }
if $help; then
    say 'Install an extracted Veil archive locally (no network or sudo).' '展開済みVeilをユーザー領域へ導入します（通信・sudo不要）。'
    printf '%s\n' 'Usage: sh install.sh [--prefix DIR] [--lang ja|en] [--activate]'
    say "Default prefix: \$HOME/.local. DIR must be absolute." "既定の保存先: \$HOME/.local。DIRは絶対パスです。"
    say '--activate: switch only a verified Veil-managed link; never overwrite another CLI.' '--activate: 検証済みVeil管理リンクだけを切替。他のCLIは上書きしません。'
    say 'Shell startup files are never edited. Read docs/cli/install.en.md.' 'シェル設定は編集しません。docs/cli/install.mdも確認してください。'
    exit 0
fi

# Reject links in managed paths, including dangling links. Do not canonicalize
# through a caller-controlled link and accidentally write outside the prefix.
safe_path() {
    check_path=$1
    while [ "$check_path" != / ]; do
        [ ! -L "$check_path" ] || return 1
        check_path=${check_path%/*}
        [ -n "$check_path" ] || check_path=/
    done
}
case $prefix in /*) ;; *) fail 'Prefix must be absolute.' '保存先は絶対パスで指定してください。' ;; esac
case "$prefix/" in */../*|*/./*|*//*|*:*|*'
'*) fail 'Unsafe prefix.' '安全に扱えない保存先です。' ;; esac
[ "$prefix" != / ] || fail 'Do not use / as prefix.' '/は保存先にできません。'
safe_path "$prefix" || fail 'Prefix contains a symbolic link.' '保存先にシンボリックリンクが含まれます。'
source_dir=$(CDPATH='' cd -P -- "$(dirname -- "$0")" && pwd -P)

if command -v sha256sum >/dev/null 2>&1; then
    checksum() { sha256sum -c SHA256SUMS >/dev/null 2>&1; }
elif command -v shasum >/dev/null 2>&1; then
    checksum() { shasum -a 256 -c SHA256SUMS >/dev/null 2>&1; }
else
    fail 'sha256sum or shasum is required.' 'sha256sum または shasum が必要です。'
fi

# Bundle member names are deliberately restricted; they are not shell input.
# Validate every path before handing the checksum list to an external utility.
verify_packet() (
    packet=$1
    [ -f "$packet/SHA256SUMS" ] && [ ! -L "$packet/SHA256SUMS" ] || exit 1
    LC_ALL=C awk '
        NF != 2 || length($1) != 64 || $1 ~ /[^0-9a-f]/ { exit 1 }
        $2 ~ /[^A-Za-z0-9_.+\/-]/ || $2 ~ /^\// || $2 ~ /(^|\/)\.\.?($|\/)/ || $2 ~ /\/\// { exit 1 }
        $2 == "-" || $2 == "SHA256SUMS" || seen[$2]++ { exit 1 }
        { required[$2] = 1; count++ }
        END { if (!count || !required["veil"] || !required["install.sh"] || !required["LICENSE-MIT"] || !required["LICENSE-APACHE"] || !required["NOTICE"] || !required["THIRD_PARTY_NOTICES.md"] || !required["third_party/manifest.json"]) exit 1 }
    ' "$packet/SHA256SUMS" || exit 1
    while IFS= read -r row || [ -n "$row" ]; do
        member=${row#*  }
        [ "$member" != "$row" ] || exit 1
        [ -f "$packet/$member" ] && safe_path "$packet/$member" || exit 1
    done < "$packet/SHA256SUMS"
    cd -P -- "$packet" || exit 1
    checksum
)
verify_packet "$source_dir" || fail 'Invalid archive contents or checksums; no activation performed.' '配布物またはチェックサムが不正です。呼出先は切り替えていません。'
[ -x "$source_dir/veil" ] || fail 'The CLI is not executable.' 'CLIの実行権限がありません。'
header=$(od -An -v -tx1 -N20 "$source_dir/veil" | tr -d ' \n')
case "$(uname -s):$(uname -m):$header" in
    Linux:x86_64:7f454c460201*)
        [ "$(printf '%s' "$header" | cut -c37-40)" = 3e00 ] || fail 'Unsupported ELF architecture.' 'ELFのCPUが対応対象外です。'
        target=x86_64-linux ;;
    Darwin:arm64:cffaedfe0c000001*) target=aarch64-darwin ;;
    *) fail 'This archive does not match this supported OS/CPU.' '配布物と対応OS・CPUが一致しません。' ;;
esac
version_output=$(VEIL_LANG=en "$source_dir/veil" --version) || fail 'Cannot run the CLI; nothing activated.' 'CLIを実行できません。呼出先は切り替えていません。'
version=${version_output#veil }
valid_version() { printf '%s\n' "$1" | LC_ALL=C awk 'NR != 1 || $0 !~ /^[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9_.+-]+)?$/ { exit 1 }'; }
if [ "$version_output" = "$version" ] || ! valid_version "$version"; then
    fail 'Invalid CLI version.' 'CLIの版表示が不正です。'
fi

store=$prefix/share/veil
bin_dir=$prefix/bin
destination=$store/$version/$target
entry=$bin_dir/veil
if ! safe_path "$destination" || ! safe_path "$bin_dir"; then
    fail 'Managed path contains a symbolic link.' '管理先にシンボリックリンクが含まれます。'
fi

# Entry checks are repeated just before activation. Activation only replaces a
# link whose literal, absolute destination is a verified packet in this store.
old_link=
if [ -L "$entry" ]; then
    old_link=$(readlink "$entry") || fail 'Cannot inspect existing link.' '既存リンクを確認できません。'
    if [ "$old_link" != "$destination/veil" ]; then
        $activate || fail 'An entry exists; use --activate only for a Veil-managed version.' '既存の呼出先があります。Veil管理の別版へ切り替える場合だけ--activateを指定してください。'
    fi
    old_packet=${old_link%/veil}
    old_parent=${old_packet%/"$target"}
    old_version=${old_parent##*/}
    if [ "$old_link" != "$store/$old_version/$target/veil" ] || ! valid_version "$old_version" || ! safe_path "$old_packet" || ! verify_packet "$old_packet"; then
        fail 'Refusing an unmanaged or damaged existing link.' '管理外または破損した既存リンクは置換しません。'
    fi
elif [ -e "$entry" ]; then
    fail 'An existing CLI occupies bin/veil; choose another prefix.' 'bin/veilに既存CLIがあります。別の保存先を選んでください。'
fi

if [ -e "$destination" ]; then
    if [ ! -d "$destination" ] || [ ! -x "$destination/veil" ] || ! verify_packet "$destination" || ! cmp -s "$source_dir/SHA256SUMS" "$destination/SHA256SUMS"; then
        fail 'This version directory exists with different contents or permissions.' '同じ版の保存先に異なる内容または権限があります。'
    fi
else
    mkdir -p -- "$store/$version" "$bin_dir" || fail 'Cannot create installation directories.' '導入先ディレクトリを作成できません。'
    stage=$(mktemp -d "$store/.install.XXXXXX") || fail 'Cannot create a staging directory.' '一時配置先を作成できません。'
    # Deliberately retain an incomplete stage on failure for inspection. Never
    # remove a caller path or an older version as part of installation.
    while IFS= read -r row || [ -n "$row" ]; do
        member=${row#*  }
        if ! mkdir -p -- "$stage/$(dirname -- "$member")" || ! cp -p -- "$source_dir/$member" "$stage/$member"; then
            fail "Copy failed. Retained staging directory: $stage" "コピーに失敗しました。一時配置先を保持: $stage"
        fi
    done < "$source_dir/SHA256SUMS"
    if ! cp -p -- "$source_dir/SHA256SUMS" "$stage/SHA256SUMS" || ! verify_packet "$stage"; then
        fail "Copy verification failed; retained: $stage" "コピー後の検証に失敗しました。保持先: $stage"
    fi
    [ ! -e "$destination" ] && [ ! -L "$destination" ] || fail 'Destination appeared during installation.' '導入中に保存先が作成されました。'
    mv -- "$stage" "$destination" || fail "Cannot complete installation; retained: $stage" "配置を完了できません。保持先: $stage"
fi

if [ -n "$old_link" ]; then
    [ -L "$entry" ] && [ "$(readlink "$entry")" = "$old_link" ] || fail 'Entry changed; refusing activation.' '呼出先が変化したため切替を拒否します。'
    if [ "$old_link" != "$destination/veil" ]; then
        temporary_link=$bin_dir/.veil-activate-$$
        [ ! -e "$temporary_link" ] && [ ! -L "$temporary_link" ] || fail 'Activation staging entry exists.' '切替用の一時呼出先が既にあります。'
        # Both forms avoid following a destination symlink to a directory.
        case $target in
            aarch64-darwin) ln -s -h -- "$destination/veil" "$temporary_link" && mv -f -h -- "$temporary_link" "$entry" ;;
            *) ln -s -T -- "$destination/veil" "$temporary_link" && mv -f -T -- "$temporary_link" "$entry" ;;
        esac || fail 'Activation failed; check the retained version and link.' '切替に失敗しました。保持した版とリンクを確認してください。'
    fi
else
    # No force: another CLI appearing in the meantime must stay intact.
    case $target in
        aarch64-darwin) ln -s -h -- "$destination/veil" "$entry" ;;
        *) ln -s -T -- "$destination/veil" "$entry" ;;
    esac || fail 'Entry already exists or cannot be created; no replacement made.' '呼出先が既にあるか作成できません。置換していません。'
fi

say "Installed: $destination" "導入先: $destination"
say "CLI entry: $entry" "CLIの呼出先: $entry"
case ":${PATH-}:" in
    *":$bin_dir:"*) say 'PATH already contains the bin directory; verify command resolution.' 'binは既にPATHにあります。呼出先を確認してください。' ;;
    *)
        say 'For zsh (~/.zshrc) or interactive bash (~/.bashrc), add this line yourself once:' 'zsh（~/.zshrc）または対話bash（~/.bashrc）へ、必要な場合だけ次の行を手動登録してください:'
        escaped_bin=$(printf '%s' "$bin_dir" | sed "s/'/'\\\\''/g")
        printf "export PATH='%s':\"\$PATH\"\n" "$escaped_bin"
        say 'For login bash use its active profile; see docs/cli/install.en.md.' 'ログインbashの登録先は利用中のprofileです。docs/cli/install.mdを参照してください。'
        ;;
esac
say 'Open a new terminal, then check: command -v veil; veil --version' '新しいターミナルで確認: command -v veil; veil --version'
say 'Startup files were not edited. Keep previous versions for manual rollback.' 'シェル設定は変更していません。以前の版は手動復帰用に保持します。'
