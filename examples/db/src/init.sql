CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table
    thingies (
        id uuid primary key default uuid_generate_v4 (),
        name varchar not null,
        num32 integer not null,
        option_num32 integer,
        num64 bigint not null,
        option_num64 bigint,
        text varchar not null,
        option_text varchar,
        custom varchar not null,
        option_custom varchar,
        multiple_custom varchar not null
    );

insert into
    thingies (
        name,
        num32,
        option_num32,
        num64,
        option_num64,
        text,
        option_text,
        custom,
        option_custom,
        multiple_custom
    )
values
    ('name1', 1, 1, 1, 1, '1', '1', 'c1', 'c1', 'c1'),
    ('name2', 2, 2, 2, 2, '2', '2', 'c2', 'c2', 'c2'),
    (
        'name3',
        3,
        null,
        3,
        3,
        '3',
        '3',
        'c3',
        null,
        'c3'
    ),
    ('name4', 4, 4, 4, 4, '4', '4', 'c4', 'c4', 'c4'),
    (
        'name5',
        5,
        null,
        5,
        5,
        '5',
        '5',
        'c5',
        'c5',
        'c5'
    ),
    ('name6', 6, 6, 6, 6, '6', '6', 'c6', 'c6', 'c6'),
    ('name7', 7, 7, 7, 7, '7', '7', 'c7', 'c7', 'c7'),
    ('name8', 8, 8, 8, 8, '8', '8', 'c8', 'c8', 'c8');
