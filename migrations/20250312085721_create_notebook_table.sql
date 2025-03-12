-- Add migration script here
create table notebook (
  id text primary key not null unique,
  name text not null,
  description text default null,
  created_at text not null default current_timestamp,
  updated_at text not null default current_timestamp
);

create index notebook_name_index on notebook(name);
-- create index for dataset as well
create index dataset_name_index on dataset(name);

create trigger if not exists notebook_updated_at_trigger
after update on notebook
begin 
    update notebook
    set updated_at = datetime('now')
    where id = new.id;
end;
