"""Base class for all ODAMP AI agents."""

import hashlib
import json
import time
from abc import ABC, abstractmethod
from typing import Any, Optional

import anthropic
import openai


class BaseAgent(ABC):
    def __init__(self, provider: str, model: str, api_key: str, max_tokens: int = 2048, temperature: float = 0.0):
        self.provider = provider
        self.model = model
        self.max_tokens = max_tokens
        self.temperature = temperature

        if provider == "anthropic":
            self.client = anthropic.Anthropic(api_key=api_key)
        elif provider == "openai":
            self.client = openai.OpenAI(api_key=api_key)
        else:
            raise ValueError(f"Unknown provider: {provider}")

    @abstractmethod
    def build_prompt(self, data: dict) -> tuple[str, str]:
        """Build (system_prompt, user_prompt) from input data."""
        ...

    @abstractmethod
    def parse_response(self, raw: str) -> dict:
        """Parse LLM response into structured output."""
        ...

    async def analyze(self, data: dict) -> dict:
        """Run the agent: build prompt → call LLM → parse → log."""
        system, user = self.build_prompt(data)
        prompt_hash = hashlib.sha256((system + user).encode()).hexdigest()

        start = time.time()
        raw_response = self._call_llm(system, user)
        latency_ms = int((time.time() - start) * 1000)

        result = self.parse_response(raw_response)

        # Log (TODO: write to DB)
        self._log({
            "agent": self.__class__.__name__,
            "model": self.model,
            "prompt_hash": prompt_hash,
            "latency_ms": latency_ms,
            "response": result,
        })

        result["model"] = self.model
        result["latency_ms"] = latency_ms
        return result

    def _call_llm(self, system: str, user: str) -> str:
        if self.provider == "anthropic":
            response = self.client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                system=system,
                messages=[{"role": "user", "content": user}],
            )
            return response.content[0].text
        else:
            response = self.client.chat.completions.create(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                messages=[
                    {"role": "system", "content": system},
                    {"role": "user", "content": user},
                ],
            )
            return response.choices[0].message.content

    def _log(self, entry: dict):
        """Log the agent call. TODO: write to ai_agent_logs table."""
        import logging
        logging.info(f"AI Agent Log: {json.dumps(entry, default=str)[:500]}")   