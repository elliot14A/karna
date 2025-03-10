-- Add migration script here
create table if not exists dataset (
    id text primary key not null unique,
    name text not null,
    file_name text not null,
    type text not null,
    description text,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    row_count integer not null,
    size integer not null
);

create trigger if not exists dataset_updated_at_trigger
after update on dataset
begin 
    update dataset
    set updated_at = datetime('now')
    where id = new.id;
end;
