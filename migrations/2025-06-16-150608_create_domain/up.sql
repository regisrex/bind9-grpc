-- Your SQL goes here
CREATE TABLE domains(
    id serial NOT NULL,
    domain text NOT NULL,
    is_deleted boolean  DEFAULT false,
    CONSTRAINT domain_pkey PRIMARY KEY (id)

)