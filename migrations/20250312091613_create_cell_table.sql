-- Add migration script here
create table cell (
  id text primary key not null unique, 
  notebook_id text not null,
  sql text not null,
  execution_status text not null default 'NotRun'
    check (execution_status in ('NotRun', 'Running', 'Success', 'Error')),
  position integer not null,
  selected_query_language text not null default 'sql'
    check (selected_query_language in ('SQL', 'GraphQL', 'NaturalLanguage')),
  result_data json default null,
  result_error text default null,
  created_at text not null default current_timestamp,
  last_run_at text default null,
  execution_time real default null,
  constraint fk_notebook_id
    foreign key (notebook_id)
    references notebook(id)
    on delete cascade,
  constraint unique_notebook_position
    unique (notebook_id, position)
);

create index cell_notebook_id_index on cell(notebook_id);
create index cell_position_index on cell(position);

-- create trigger to update last_run_at
create trigger if not exists update_cell_last_run_at_trigger
after update on cell
begin 
    update cell
    set last_run_at = datetime('now')
    where id = new.id;
end;
