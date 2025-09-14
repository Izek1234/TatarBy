import os
import torch
from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse, JSONResponse
from pydantic import BaseModel
from typing import Optional
from PIL import Image
import io
import base64
from dotenv import load_dotenv
from diffusers import StableDiffusionPipeline
from transformers import pipeline
from fastapi.middleware.cors import CORSMiddleware


load_dotenv()

HF_TOKEN = os.getenv("HF_TOKEN", "").strip() or None
MODEL_ID = os.getenv("MODEL_ID", "runwayml/stable-diffusion-v1-5")
NEGATIVE_PROMPT = os.getenv("NEGATIVE_PROMPT", "low quality, blurry, cartoon, deformed, ugly")
NUM_INFERENCE_STEPS = int(os.getenv("NUM_INFERENCE_STEPS", 50))
GUIDANCE_SCALE = float(os.getenv("GUIDANCE_SCALE", 7.5))
WIDTH = int(os.getenv("WIDTH", 512))
HEIGHT = int(os.getenv("HEIGHT", 512))
SEED = int(os.getenv("SEED", 42))
USE_HALF_PRECISION = os.getenv("USE_HALF_PRECISION", "true").lower() == "true"
USE_8BIT_QUANTIZATION = os.getenv("USE_8BIT_QUANTIZATION", "true").lower() == "true"
DEVICE = os.getenv("DEVICE", "cuda" if torch.cuda.is_available() else "cpu")

print(f"🧠 Loading model: {MODEL_ID} on {DEVICE}")
print(f"⚡ Half precision: {USE_HALF_PRECISION}, 8-bit quant: {USE_8BIT_QUANTIZATION}")

pipe = None

try:
    torch_dtype = torch.float16 if USE_HALF_PRECISION and DEVICE == "cuda" else torch.float32

    quantization_config = None
    if USE_8BIT_QUANTIZATION:
        from transformers import BitsAndBytesConfig
        quantization_config = BitsAndBytesConfig(
            load_in_8bit=True,
            llm_int8_enable_fp32_cpu_offload=True
        )

    pipe = StableDiffusionPipeline.from_pretrained(
        MODEL_ID,
        torch_dtype=torch_dtype,
        safety_checker=None,  
        use_auth_token=HF_TOKEN,
        quantization_config=quantization_config,
        device_map="auto",
        cache_dir="./model_cache"
    )

    if DEVICE == "cuda" and USE_HALF_PRECISION:
        pipe = pipe.to("cuda")

except Exception as e:
    print(f"❌ Failed to load model: {e}")
    raise RuntimeError("Could not load Stable Diffusion model.")

class ImageGenerationRequest(BaseModel):
    prompt: str
    negative_prompt: Optional[str] = NEGATIVE_PROMPT
    num_inference_steps: Optional[int] = NUM_INFERENCE_STEPS
    guidance_scale: Optional[float] = GUIDANCE_SCALE
    width: Optional[int] = WIDTH
    height: Optional[int] = HEIGHT
    seed: Optional[int] = SEED
    return_base64: Optional[bool] = True  

class ImageGenerationResponse(BaseModel):
    image: Optional[str] = None  
    success: bool
    message: str

app = FastAPI(title="TaT Diffusion v1.5 API", version="1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/")
async def root():
    return {
        "status": "OK",
        "model": MODEL_ID,
        "device": DEVICE,
        "half_precision": USE_HALF_PRECISION,
        "quantized": USE_8BIT_QUANTIZATION
    }

@app.post("/generate", response_model=ImageGenerationResponse)
async def generate_image(request: ImageGenerationRequest):
    try:
        generator = torch.Generator(device=DEVICE).manual_seed(request.seed)

        image = pipe(
            prompt=request.prompt,
            negative_prompt=request.negative_prompt,
            num_inference_steps=request.num_inference_steps,
            guidance_scale=request.guidance_scale,
            width=request.width,
            height=request.height,
            generator=generator,
        ).images[0]

        buffer = io.BytesIO()
        image.save(buffer, format="PNG")
        buffer.seek(0)

        if request.return_base64:
            img_str = base64.b64encode(buffer.getvalue()).decode("utf-8")
            return ImageGenerationResponse(
                image=img_str,
                success=True,
                message="Image generated successfully"
            )
        else:
            return StreamingResponse(
                buffer,
                media_type="image/png",
                headers={"Content-Disposition": "attachment; filename=image.png"}
            )

    except Exception as e:
        print(f"❌ Generation error: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to generate image: {str(e)}")


@app.get("/health")
async def health_check():
    return {"status": "healthy", "model_loaded": pipe is not None}