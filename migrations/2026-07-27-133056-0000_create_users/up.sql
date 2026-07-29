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
CREATE TABLE urls (
  database_id INT PRIMARY KEY AUTO INCREMENT,
  short_code VARCHAR(4),
  long_url VARCHAR(2048) NOT NULL,
  created_by UUID NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_urls FOREIGN KEY (created_by) REFERENCES users(id)
);
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
  CONSTRAINT fk_url_analytics FOREIGN KEY (short_code) REFERENCES urls(short_code)
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
  CONSTRAINT fk_transactions FOREIGN KEY (user_id) REFERENCES users(id)
);
-- CREATE TABLE subscribtions();
 