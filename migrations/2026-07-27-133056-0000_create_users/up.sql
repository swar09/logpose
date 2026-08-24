CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ 
BEGIN 
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  first_name VARCHAR(100) NOT NULL,
  last_name VARCHAR(100) NOT NULL,
  username VARCHAR(30) NOT NULL,
  email VARCHAR(100) NOT NULL,
  hashed_password VARCHAR(128) NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE UNIQUE INDEX idx_users_lower_email ON users(LOWER(email));
CREATE UNIQUE INDEX idx_users_lower_username ON users(LOWER(username));
CREATE TRIGGER update_users_updated_at BEFORE
UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE urls (
  database_id SERIAL PRIMARY KEY,
  short_code VARCHAR(30) UNIQUE,
  long_url VARCHAR(2048) NOT NULL,
  created_by UUID,
  guest_id UUID,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_urls FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_urls_created_by ON urls(created_by);
CREATE INDEX idx_urls_guest_id ON urls(guest_id);
CREATE INDEX idx_urls_expires_at ON urls(expires_at);
CREATE TRIGGER update_urls_updated_at BEFORE
UPDATE ON urls FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE url_analytics(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  short_code VARCHAR(30),
  clicked_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  ip_address VARCHAR(45) NOT NULL,
  user_agent VARCHAR(1024),
  browser VARCHAR(100),
  device VARCHAR(100),
  country_code VARCHAR(10),
  referer VARCHAR(2048),
  CONSTRAINT fk_url_analytics FOREIGN KEY (short_code) REFERENCES urls(short_code) ON DELETE SET NULL
);
CREATE INDEX idx_url_analytics_short_code ON url_analytics(short_code);

CREATE TYPE billing_interval AS ENUM ('monthly', 'yearly', 'lifetime', 'one_time');
CREATE TYPE subscription_status AS ENUM ('created', 'active', 'past_due', 'canceled', 'expired');
CREATE TYPE payment_status AS ENUM ('created', 'authorized', 'captured', 'failed', 'refunded');
CREATE TYPE webhook_processing_status AS ENUM ('pending', 'processed', 'failed', 'ignored');

CREATE TABLE plans (
  id SERIAL PRIMARY KEY,
  name VARCHAR(50) NOT NULL UNIQUE,
  code VARCHAR(50) NOT NULL UNIQUE,
  description TEXT,
  amount INT NOT NULL,
  currency VARCHAR(3) NOT NULL DEFAULT 'INR',
  interval billing_interval NOT NULL DEFAULT 'monthly',
  max_urls_limit INT NOT NULL DEFAULT 50,
  custom_alias_allowed BOOLEAN NOT NULL DEFAULT FALSE,
  analytics_retention_days INT NOT NULL DEFAULT 7,
  features JSONB NOT NULL DEFAULT '{}'::jsonb,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE TRIGGER update_plans_updated_at BEFORE
UPDATE ON plans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE user_subscriptions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  plan_id INT NOT NULL,
  status subscription_status NOT NULL DEFAULT 'created',
  razorpay_subscription_id VARCHAR(100) UNIQUE,
  razorpay_customer_id VARCHAR(100),
  current_period_start TIMESTAMPTZ,
  current_period_end TIMESTAMPTZ,
  cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
  canceled_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_user_subscriptions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_user_subscriptions_plan FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE RESTRICT
);
CREATE INDEX idx_user_subscriptions_user_id ON user_subscriptions(user_id);
CREATE INDEX idx_user_subscriptions_status ON user_subscriptions(status);
CREATE TRIGGER update_user_subscriptions_updated_at BEFORE
UPDATE ON user_subscriptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE payments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  plan_id INT NOT NULL,
  subscription_id UUID,
  amount INT NOT NULL,
  currency VARCHAR(3) NOT NULL DEFAULT 'INR',
  status payment_status NOT NULL DEFAULT 'created',
  razorpay_order_id VARCHAR(100) NOT NULL UNIQUE,
  razorpay_payment_id VARCHAR(100) UNIQUE,
  razorpay_signature TEXT,
  error_code VARCHAR(100),
  error_description TEXT,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  CONSTRAINT fk_payments_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_payments_plan FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE RESTRICT,
  CONSTRAINT fk_payments_subscription FOREIGN KEY (subscription_id) REFERENCES user_subscriptions(id) ON DELETE SET NULL
);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_razorpay_order ON payments(razorpay_order_id);
CREATE TRIGGER update_payments_updated_at BEFORE
UPDATE ON payments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE webhook_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id VARCHAR(100) NOT NULL UNIQUE,
  event_type VARCHAR(100) NOT NULL,
  status webhook_processing_status NOT NULL DEFAULT 'pending',
  payload JSONB NOT NULL,
  error_log TEXT,
  processed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE INDEX idx_webhook_events_event_id ON webhook_events(event_id);