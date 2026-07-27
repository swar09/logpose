CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  first_name VARCHAR(100) NOT NULL,
  last_name VARCHAR(100) NOT NULL,
  username VARCHAR(30) UNIQUE NOT NULL,
  email VARCHAR(100) UNIQUE NOT NULL,
  hashed_password VARCHAR(128) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE urls (
  short_code PRIMARY KEY VARCHAR(7),
  long_url VARCHAR(2048) NOT NULL,
  created_by UUID , -- foreign key to users 
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE url_analytics(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  short_code VARCHAR(2048) , -- foreign key urls 
  clicked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  ip_address VARCHAR(45) , 
  user_agent VARCHAR(1024),
  browser VARCHAR(),
  device VARCHAR(),
  country_code VARCHAR(),
  referer VARCHAR(),
);
CREATE TABLE transactions(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

);
-- CREATE TABLE subscribtions();
