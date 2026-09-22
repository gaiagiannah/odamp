import os
from dotenv import load_dotenv

load_dotenv()

class Settings:
    def __init__(self):
        self.database_url = os.getenv("DATABASE_URL", "postgresql://odamp:odamp_dev@localhost:5432/odamp")
        self.llm_provider = os.getenv("LLM_PROVIDER", "anthropic")
        self.llm_api_key = os.getenv("LLM_API_KEY", "")
        self.llm_model = os.getenv("LLM_MODEL", "claude-sonnet-4-20250514")
        self.llm_max_tokens = int(os.getenv("LLM_MAX_TOKENS", "2048"))
        self.llm_temperature = float(os.getenv("LLM_TEMPERATURE", "0"))

settings = Settings()   