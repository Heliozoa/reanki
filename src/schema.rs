// @generated automatically by Diesel CLI.

diesel::table! {
    cards (id) {
        id -> Nullable<BigInt>,
        nid -> BigInt,
        did -> BigInt,
        ord -> BigInt,
        #[sql_name = "mod"]
        mod_ -> BigInt,
        usn -> BigInt,
        #[sql_name = "type"]
        type_ -> BigInt,
        queue -> BigInt,
        due -> BigInt,
        ivl -> BigInt,
        factor -> BigInt,
        reps -> BigInt,
        lapses -> BigInt,
        left -> BigInt,
        odue -> BigInt,
        odid -> BigInt,
        flags -> BigInt,
        data -> Text,
    }
}

diesel::table! {
    col (id) {
        id -> Nullable<BigInt>,
        crt -> BigInt,
        #[sql_name = "mod"]
        mod_ -> BigInt,
        scm -> BigInt,
        ver -> BigInt,
        dty -> BigInt,
        usn -> BigInt,
        ls -> BigInt,
        conf -> Text,
        models -> Text,
        decks -> Text,
        dconf -> Text,
        tags -> Text,
    }
}

diesel::table! {
    graves (rowid) {
        rowid -> BigInt,
        usn -> BigInt,
        oid -> BigInt,
        #[sql_name = "type"]
        type_ -> BigInt,
    }
}

diesel::table! {
    notes (id) {
        id -> Nullable<BigInt>,
        guid -> Text,
        mid -> BigInt,
        #[sql_name = "mod"]
        mod_ -> BigInt,
        usn -> BigInt,
        tags -> Text,
        flds -> Text,
        sfld -> BigInt,
        csum -> BigInt,
        flags -> BigInt,
        data -> Text,
    }
}

diesel::table! {
    revlog (id) {
        id -> Nullable<BigInt>,
        cid -> BigInt,
        usn -> BigInt,
        ease -> BigInt,
        ivl -> BigInt,
        lastIvl -> BigInt,
        factor -> BigInt,
        time -> BigInt,
        #[sql_name = "type"]
        type_ -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(cards, col, graves, notes, revlog,);
