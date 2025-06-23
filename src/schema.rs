// @generated automatically by Diesel CLI.

diesel::table! {
    cards (id) {
        id -> Nullable<Integer>,
        nid -> Integer,
        did -> Integer,
        ord -> Integer,
        #[sql_name = "mod"]
        mod_ -> Integer,
        usn -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
        queue -> Integer,
        due -> Integer,
        ivl -> Integer,
        factor -> Integer,
        reps -> Integer,
        lapses -> Integer,
        left -> Integer,
        odue -> Integer,
        odid -> Integer,
        flags -> Integer,
        data -> Text,
    }
}

diesel::table! {
    col (id) {
        id -> Nullable<Integer>,
        crt -> Integer,
        #[sql_name = "mod"]
        mod_ -> Integer,
        scm -> Integer,
        ver -> Integer,
        dty -> Integer,
        usn -> Integer,
        ls -> Integer,
        conf -> Text,
        models -> Text,
        decks -> Text,
        dconf -> Text,
        tags -> Text,
    }
}

diesel::table! {
    graves (rowid) {
        rowid -> Integer,
        usn -> Integer,
        oid -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
    }
}

diesel::table! {
    notes (id) {
        id -> Nullable<Integer>,
        guid -> Text,
        mid -> Integer,
        #[sql_name = "mod"]
        mod_ -> Integer,
        usn -> Integer,
        tags -> Text,
        flds -> Text,
        sfld -> Integer,
        csum -> Integer,
        flags -> Integer,
        data -> Text,
    }
}

diesel::table! {
    revlog (id) {
        id -> Nullable<Integer>,
        cid -> Integer,
        usn -> Integer,
        ease -> Integer,
        ivl -> Integer,
        lastIvl -> Integer,
        factor -> Integer,
        time -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cards,
    col,
    graves,
    notes,
    revlog,
);
