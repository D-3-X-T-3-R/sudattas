FROM mysql:latest

# Environment variables
ENV MYSQL_DATABASE=SUDATTAS
ENV MYSQL_ROOT_PASSWORD=12345678

ADD sql_dump/01_schema.sql /docker-entrypoint-initdb.d/
# Remove this later
ADD sql_dump/02_data.sql /docker-entrypoint-initdb.d/  

RUN chown -R mysql:mysql /docker-entrypoint-initdb.d/