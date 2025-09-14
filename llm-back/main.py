from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
from huggingface_hub import login
import os
from dotenv import load_dotenv

load_dotenv() 

HF_TOKEN = os.getenv("HF_TOKEN")
if not HF_TOKEN:
    raise RuntimeError("❌ HF_TOKEN не найден в .env")

MODEL_NAME = os.getenv("MODEL_NAME", None)

MAX_NEW_TOKENS = int(os.getenv("MAX_NEW_TOKENS", 512))
TEMPERATURE = float(os.getenv("TEMPERATURE", 0.7))
DO_SAMPLE = os.getenv("DO_SAMPLE", "true").lower() == "true"
PAD_TOKEN_ID_STR = os.getenv("PAD_TOKEN_ID", "eos")

LOAD_IN_8BIT = os.getenv("LOAD_IN_8BIT", "true").lower() == "true"
LOAD_IN_4BIT = os.getenv("LOAD_IN_4BIT", "false").lower() == "true"
LLM_INT8_ENABLE_FP32_CPU_OFFLOAD = os.getenv("LLM_INT8_ENABLE_FP32_CPU_OFFLOAD", "true").lower() == "true"

DEVICE = os.getenv("DEVICE", "cuda" if torch.cuda.is_available() else "cpu")

print(f"🧠 Модель: {MODEL_NAME}")
print(f"🔧 Устройство: {DEVICE}")
print(f"⚡ Квантование: 8-bit={LOAD_IN_8BIT}, 4-bit={LOAD_IN_4BIT}")

login(token=HF_TOKEN)

tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, token=HF_TOKEN)

tokenizer.chat_template = """[INST] {{ messages[0]['content'] }} [/INST]"""

model = None

try:
    quantization_config = None

    if LOAD_IN_4BIT:
        quantization_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_use_double_quant=True,
            bnb_4bit_compute_dtype=torch.bfloat16
        )
    elif LOAD_IN_8BIT:
        quantization_config = BitsAndBytesConfig(
            load_in_8bit=True,
            llm_int8_enable_fp32_cpu_offload=LLM_INT8_ENABLE_FP32_CPU_OFFLOAD
        )

    device_map = "auto" if DEVICE == "cuda" else "cpu"

    model = AutoModelForCausalLM.from_pretrained(
        MODEL_NAME,
        device_map=device_map,
        quantization_config=quantization_config,
        torch_dtype=torch.float16 if DEVICE == "cuda" and not LOAD_IN_4BIT else torch.float32,
        token=HF_TOKEN
    )
    print("✅ Модель успешно загружена")

except Exception as e:
    print(f"❌ Ошибка при загрузке модели: {e}")
    print("⏳ Загружаем модель на CPU без квантования...")
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_NAME,
        device_map="cpu",
        torch_dtype=torch.float32,
        token=HF_TOKEN
    )

if model is None:
    raise RuntimeError("❌ Не удалось загрузить модель!")

print(f"🌐 Модель работает на устройстве: {model.device}")

pad_token_id = tokenizer.eos_token_id if PAD_TOKEN_ID_STR == "eos" else int(PAD_TOKEN_ID_STR)

class ChatMessage(BaseModel):
    content: str

class ChatRequest(BaseModel):
    messages: List[ChatMessage]
    max_new_tokens: Optional[int] = MAX_NEW_TOKENS
    temperature: Optional[float] = TEMPERATURE

class ChatResponse(BaseModel):
    response: str

app = FastAPI(title="TatarGPT API", version="1.0")

@app.post("/chat", response_model=ChatResponse)
async def chat_endpoint(request: ChatRequest):

    if not request.messages or not request.messages[0].content.strip():
        raise HTTPException(status_code=400, detail="Сообщение не может быть пустым")

    user_input = request.messages[0].content

    prompt = f"[INST] {user_input} [/INST]"

    try:
        inputs = tokenizer(prompt, return_tensors="pt").to(model.device)

        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=request.max_new_tokens,
                temperature=request.temperature,
                do_sample=DO_SAMPLE,
                pad_token_id=pad_token_id,
                eos_token_id=tokenizer.eos_token_id,
            )

        response_ids = outputs[0][inputs.input_ids.shape[1]:]
        response_text = tokenizer.decode(response_ids, skip_special_tokens=True)

        return ChatResponse(response=response_text)

    except Exception as e:
        print(f"Ошибка при генерации: {e}")
        raise HTTPException(status_code=500, detail=f"Ошибка генерации: {str(e)}")


@app.get("/")
async def root():
    return {
        "status": "OK",
        "model": MODEL_NAME,
        "device": str(model.device),
        "loaded_with_8bit": LOAD_IN_8BIT,
        "loaded_with_4bit": LOAD_IN_4BIT,
        "temperature": TEMPERATURE,
        "max_new_tokens": MAX_NEW_TOKENS,
        "do_sample": DO_SAMPLE
    }

