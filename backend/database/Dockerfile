FROM mysql:latest

# Environment variables
ENV MYSQL_DATABASE=sudattas
ENV MYSQL_ROOT_PASSWORD=12345678

ADD sql_dump/schema.sql /docker-entrypoint-initdb.d/

RUN chown -R mysql:mysql /docker-entrypoint-initdb.d/