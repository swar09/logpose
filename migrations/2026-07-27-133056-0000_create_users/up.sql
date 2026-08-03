CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  first_name VARCHAR(100) NOT NULL,
  last_name VARCHAR(100) NOT NULL,
  username VARCHAR(30) UNIQUE NOT NULL,
  email VARCHAR(100) UNIQUE NOT NULL,
  hashed_password VARCHAR(128) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE urls (
  database_id SERIAL PRIMARY KEY ,
  short_code VARCHAR(4) UNIQUE,
  long_url VARCHAR(2048) NOT NULL,
  created_by UUID NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_urls FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TRIGGER update_urls_updated_at
    BEFORE UPDATE ON urls
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE url_analytics(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  short_code VARCHAR(4) NOT NULL,
  clicked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  ip_address VARCHAR(45) NOT NULL,
  user_agent VARCHAR(1024),
  browser VARCHAR(100),
  device VARCHAR(100),
  country_code VARCHAR(10),
  referer VARCHAR(2048),
  CONSTRAINT fk_url_analytics FOREIGN KEY (short_code) REFERENCES urls(short_code) ON DELETE CASCADE
);

CREATE TYPE transaction_status AS ENUM ('pending', 'success', 'failed');
CREATE TABLE transactions(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  amount INT NOT NULL,
  currency_code VARCHAR(5) NOT NULL,
  status transaction_status NOT NULL,
  reference_id UUID ,
  timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_transactions FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
-- CREATE TABLE subscribtions();

 