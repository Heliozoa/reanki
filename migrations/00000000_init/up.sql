-- collections
CREATE TABLE col (
    id integer primary key,
    crt bigint not null,
    mod bigint not null,
    scm bigint not null,
    ver bigint not null,
    dty bigint not null,
    usn bigint not null,
    ls bigint not null,
    conf text not null,
    models text not null,
    decks text not null,
    dconf text not null,
    tags text not null
);
CREATE TABLE notes (
    id integer primary key,
    guid text not null,
    mid bigint not null,
    mod bigint not null,
    usn bigint not null,
    tags text not null,
    flds text not null,
    sfld bigint not null,
    csum bigint not null,
    flags bigint not null,
    data text not null
);
CREATE TABLE cards (
    id integer primary key,
    nid bigint not null,
    did bigint not null,
    ord bigint not null,
    mod bigint not null,
    usn bigint not null,
    type bigint not null,
    queue bigint not null,
    due bigint not null,
    ivl bigint not null,
    factor bigint not null,
    reps bigint not null,
    lapses bigint not null,
    left bigint not null,
    odue bigint not null,
    odid bigint not null,
    flags bigint not null,
    data text not null
);
-- not used by us but required
CREATE TABLE revlog (
    id integer primary key,
    cid bigint not null,
    usn bigint not null,
    ease bigint not null,
    ivl bigint not null,
    lastIvl bigint not null,
    factor bigint not null,
    time bigint not null,
    type bigint not null
);
-- not used by us but required
CREATE TABLE graves (
    usn bigint not null,
    oid bigint not null,
    type bigint not null
);
CREATE INDEX ix_notes_usn on notes (usn);
CREATE INDEX ix_cards_usn on cards (usn);
CREATE INDEX ix_revlog_usn on revlog (usn);
CREATE INDEX ix_cards_nid on cards (nid);
CREATE INDEX ix_cards_sched on cards (did, queue, due);
CREATE INDEX ix_revlog_cid on revlog (cid);
CREATE INDEX ix_notes_csum on notes (csum);