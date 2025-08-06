import vertexai
from vertexai.generative_models import GenerativeModel, GenerationConfig, grounding
from vertexai.generative_models import Tool
from vertexai import rag
from vertexai.rag.utils.resources import TransformationConfig, ChunkingConfig
from typing import Generator, List, Optional
from data_classes.common_classes import Agent, Message, Language, StreamEvent
from dotenv import load_dotenv
from vertexai.preview.generative_models import Content, Part
from werkzeug.datastructures import FileStorage 
import tempfile
from werkzeug.utils import secure_filename
from vertexai.rag.utils.resources import RagFile
from google.cloud.aiplatform_v1.services.vertex_rag_data_service.pagers import ListRagFilesPager
import os
from vertexai.rag.utils.resources import RagCorpus
from vertexai.tuning import sft
from google.cloud import storage
import logging
from datetime import datetime
from typing import Dict, Any
from constants.separators import STARTING_SEPARATOR, ENDING_SEPARATOR
import asyncio
logger = logging.getLogger(__name__)
from google.genai import Client
from google.genai import types

load_dotenv()

PROJECT_ID = os.getenv("GOOGLE_PROJECT_ID")

RAG_LOCATION = os.getenv("GOOGLE_RAG_LOCATION")
if not PROJECT_ID:
    raise ValueError("GOOGLE_PROJECT_ID is not set")
if not RAG_LOCATION:
    raise ValueError("GOOGLE_RAG_LOCATION is not set")

# RAG_CORPUS_NAME = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/6917529027641081856"

TRANSFORMATION_CONFIG: TransformationConfig = TransformationConfig(
    chunking_config=ChunkingConfig(
        chunk_size=1024,
        chunk_overlap=200,
    ),
)

vertexai.init(project=PROJECT_ID, location=RAG_LOCATION)

# Here is you:
    
system_prompt = """
You are:{{agent_name}}
Here is the your persona:
{{agent_persona}}

RULES:
- If you do not know the answer , say sorry and say you don't know, don't hallucinate.
- Don't need to add citation numbers to your response.
- response in markdown format
- all quotes should be in blockquote
- ALWAYS respond in {{base_language}}
{{context}}
"""

def generate_gemini_response(
    agent: Agent,
    messages: List[Message], 
    context: Optional[str] = None,
    stream: bool = False
) -> str | Generator[StreamEvent, None, None]:
    """
    Sends a query to a Gemini model, grounded with a Vertex AI Search data store.

    Args:
        user_query: The user's question or input.

    Returns:
        The text response from the Gemini model, potentially with citations.
    """
    if not agent:
        raise Exception("Agent is required")
    base_language = getattr(agent, "language", Language.VI.value)
    base_model = getattr(agent, "model", "gemini-2.0-flash-001")
    base_temperature = getattr(agent, "temperature", 1)
    agent_name = getattr(agent, "name", "Sư Tam Vô AI")
    
    base_system_prompt = system_prompt
    base_system_prompt = base_system_prompt.replace("{{agent_name}}", agent_name)
    base_system_prompt = base_system_prompt.replace("{{STARTING_SEPARATOR}}", STARTING_SEPARATOR)
    base_system_prompt = base_system_prompt.replace("{{ENDING_SEPARATOR}}", ENDING_SEPARATOR)
    base_system_prompt = base_system_prompt.replace("{{base_language}}", "Vietnamese" if base_language == Language.VI.value else "English")
    base_system_prompt = base_system_prompt.replace("{{agent_persona}}", getattr(agent, "system_prompt", ""))
    base_system_prompt = base_system_prompt.replace("{{context}}", context or "")
    corpus_id = getattr(agent, "corpus_id", None)
    
    rag_retrieval_tool_2 = None
    
    if corpus_id:
        RAG_CORPUS_NAME = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
        rag_retrieval_tool_2 = types.Tool(
            retrieval=types.Retrieval(
            vertex_rag_store=types.VertexRagStore(
                    rag_resources=[
                        types.VertexRagStoreRagResource(
                            rag_corpus=RAG_CORPUS_NAME,
                        )
                    ],
                    rag_retrieval_config=types.RagRetrievalConfig(
                        top_k=20,
                        filter=types.RagRetrievalConfigFilter(
                            vector_distance_threshold=0.7,
                        ),
                    ),
                ),
            )
        )
    user_query = messages[-1].content
    client = Client(
        vertexai=True,
        project=PROJECT_ID,
        location=RAG_LOCATION
    )
    history = []
    if messages:
        history = [types.Content(role=x.role, parts=[types.Part(text=x.content)]) for x in messages]
    
    thinking_config = None
    if base_model.startswith("gemini-2.5"):
        thinking_config = types.ThinkingConfig(
            thinking_budget=-1,
            include_thoughts=True,
        )
    tools = []
    if rag_retrieval_tool_2:
        tools.append(rag_retrieval_tool_2)
    print(f"tools: {len(tools)}")
    try:
        if stream:
            generator = client.models.generate_content_stream(
                model=base_model,
                contents=history,
                config=types.GenerateContentConfig(
                    system_instruction=base_system_prompt,
                    tools=tools,
                    response_mime_type="text/plain",
                    response_modalities=["TEXT"],
                    max_output_tokens=8192,
                    temperature=base_temperature,
                    top_p = 1,
                    top_k=40,
                    thinking_config=thinking_config,
                )
            )
            def generate():
                full_response = ""
                for chunk in generator:
                    if chunk:
                        full_response += chunk.text or ""
                        if chunk.candidates and chunk.candidates[0] and chunk.candidates[0].content:
                            for part in chunk.candidates[0].content.parts or []:
                                if part and part.text:
                                    if part.thought:
                                        yield StreamEvent(type="thought", data=part.text or "")
                                    else:
                                        yield StreamEvent(type="text", data=part.text or "")
                
                
                yield StreamEvent(type="end_of_stream", data="", metadata=None)
            return generate()
        else:
            response = client.models.generate_content(
                model=base_model,
                # model='projects/566310375218/locations/us-central1/endpoints/3767644817853513728',
                contents=[
                    types.Content(
                        role="user",
                        parts=[types.Part(text=user_query)]
                    )
                ],
                config=types.GenerateContentConfig(
                    system_instruction=base_system_prompt,
                    tools=tools,
                    response_mime_type="text/plain",
                    response_modalities=["TEXT"],
                    max_output_tokens=8192,
                    temperature=base_temperature,
                    top_p = 1,
                    top_k=40,
                    thinking_config=thinking_config,
                )
            )
            # print(response)
            
            # Debug: Print response length
            # print(f"DEBUG: Response length: {len(response.text)} characters, approximately {len(response.text.split())} words")
            
            # text_with_citations = add_citations(response)
            # print("*"*100)
            # print(text_with_citations)
            # print("*"*100)
            return response.text or ""
    except Exception as e:
        print(f"Error sending grounded message to Gemini: {e}")
        return "Sorry, I couldn't process your request with the knowledge base."

def add_citations(response):
    text = response.text
    supports = response.candidates[0].grounding_metadata.grounding_supports
    chunks = response.candidates[0].grounding_metadata.grounding_chunks

    # Save json file with utf-8 encoding
    # with open("response.json", "w", encoding="utf-8") as f:
    #     json.dump(response.to_dict(), f, ensure_ascii=False)
    
    # Sort supports by start_index in descending order to avoid shifting issues when inserting
    sorted_supports = sorted(supports, key=lambda s: s.segment.start_index, reverse=True)

    for support in sorted_supports:
        index = text.find(support.segment.text)
        last_index = index + len(support.segment.text)
        if support.grounding_chunk_indices:
            # Create citation string like [1](link1), [2](link2)
            citation_links = []
            for i in support.grounding_chunk_indices:
                if i < len(chunks):
                    uri = chunks[i].retrieved_context.uri.replace("gs://", "https://storage.cloud.google.com/")
                    citation_links.append(f"[{i + 1}]({uri})")

            citation_string = " " + ", ".join(citation_links)
            text = text[:last_index] + citation_string + text[last_index:]

    return text

def add_file(file: FileStorage, corpus_id: str) -> str:
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    if file.filename == '':
        return "File name is required"
    if file:
        # Create a temporary file to save the uploaded content
        # The upload_file function expects a local file path
        temp_file_path: str = ""
        try:
            # Get the original filename and ensure it has an extension
            original_filename = file.filename or "uploaded_file"
            display_name: str = secure_filename(original_filename)
            
            # Extract file extension
            file_extension = ""
            if "." in original_filename:
                file_extension = "." + original_filename.split(".")[-1]
            
            # Create temporary file with proper extension
            with tempfile.NamedTemporaryFile(delete=False, suffix=file_extension) as temp_file:
                file.save(temp_file.name)
                temp_file_path = temp_file.name

            rag_file: RagFile = rag.upload_file(
                corpus_name=full_corpus_path,
                display_name=display_name,
                path=temp_file_path,
                transformation_config=TRANSFORMATION_CONFIG,
            )
            return f"File '{display_name}' uploaded successfully to RagCorpus. RagFile ID: {rag_file.name}"
        except Exception as e:
            print(f"Error uploading file: {e}")
            raise Exception(f"Error uploading file: {e}")
        finally:
            if temp_file_path and os.path.exists(temp_file_path):
                try:
                    os.remove(temp_file_path)  # Clean up the temporary file
                except OSError as e:
                    print(f"Warning: Could not remove temporary file {temp_file_path}: {e}")
                
async def add_file_async(file: FileStorage, corpus_id: str) -> str:
    """Async wrapper for add_file function to support concurrent uploads"""
    loop = asyncio.get_event_loop()
    # Run the synchronous add_file function in a thread pool executor
    return await loop.run_in_executor(None, add_file, file, corpus_id)

async def upload_temp_file_async(temp_file_path: str, display_name: str, corpus_id: str) -> str:
    """Async function to upload a file from a temporary path to RAG corpus"""
    loop = asyncio.get_event_loop()
    
    def upload_temp_file():
        full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
        try:
            rag_file: RagFile = rag.upload_file(
                corpus_name=full_corpus_path,
                display_name=display_name,
                path=temp_file_path,
                transformation_config=TRANSFORMATION_CONFIG,
            )
            return f"File '{display_name}' uploaded successfully to RagCorpus. RagFile ID: {rag_file.name}"
        except Exception as e:
            print(f"Error uploading file: {e}")
            raise Exception(f"Error uploading file: {e}")
    
    # Run the upload in a thread pool executor
    return await loop.run_in_executor(None, upload_temp_file)

def remove_file(file_id: str, corpus_id: str) -> str:
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    file_path = f"{full_corpus_path}/ragFiles/{file_id}"
    rag.delete_file(file_path, full_corpus_path)
    return f"File '{file_id}' deleted successfully from RagCorpus."

def get_files(corpus_id: str) -> ListRagFilesPager:
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    return rag.list_files(full_corpus_path)

def read_one_file(file_id: str, corpus_id: str) -> RagFile:
    """
    This one can be improved by upload to gg storage and return the url
    """
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    file_path = f"{full_corpus_path}/ragFiles/{file_id}"
    return rag.get_file(file_path)

def add_corpus(display_name: str) -> RagCorpus:
    corpus: RagCorpus = rag.create_corpus(display_name=display_name)
    return corpus

def get_corpus(corpus_id: str) -> RagCorpus:
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    return rag.get_corpus(full_corpus_path)

def delete_corpus(corpus_id: str) -> str:
    full_corpus_path = f"projects/{PROJECT_ID}/locations/{RAG_LOCATION}/ragCorpora/{corpus_id}"
    rag.delete_corpus(full_corpus_path)
    return f"Corpus '{corpus_id}' deleted successfully."

def list_corpora():
    """List all RAG corpora in the project"""
    try:
        corpora_pager = rag.list_corpora(page_size=100)
        return list(corpora_pager) if corpora_pager else []
    except Exception as e:
        logger.error(f"Error listing corpora: {e}")
        return []

def upload_to_gcs(file_path: str, bucket_name: str) -> str:
    """Upload a file to Google Cloud Storage"""
    storage_client = storage.Client()
    bucket = storage_client.bucket(bucket_name)
    file_name = file_path.split("/")[-1]
    final_file_path = f"jsonl/{file_name}"
    blob = bucket.blob(final_file_path)
    blob.upload_from_filename(file_path)
    # should return gs://cloud-samples-data/training-file.jsonl
    return f"gs://{bucket_name}/{final_file_path}"

def create_fine_tuning_job(
    training_data_path: str,
    base_model: str = "gemini-2.5-flash",
    model_display_name: Optional[str] = None,
    hyperparameters: Optional[Dict[str, Any]] = None
) -> Dict[str, Any]:
    """
    Create a fine-tuning job using Google Vertex AI.
    
    Args:
        training_data_path: Path to the JSONL training data file
        base_model: Base model to fine-tune (default: gemini-1.5-flash-001)
        model_display_name: Display name for the fine-tuned model
        hyperparameters: Optional hyperparameters for training
    
    Returns:
        Dictionary containing job information
    """
    try:
        # The dataset can be a JSONL file on Google Cloud Storage
        if not model_display_name:
            model_display_name = f"gemini_fine_tuned_model_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        sft_tuning_job = sft.train(
            source_model=base_model,
            train_dataset=training_data_path,
            tuned_model_display_name=model_display_name,
        )
        return {
            "job_name": sft_tuning_job.name,
            "display_name": model_display_name,
            "base_model": base_model,
            "status": sft_tuning_job.state.name,
            "created_at": sft_tuning_job.create_time.isoformat(),
            "training_data_path": training_data_path,
            "hyperparameters": hyperparameters
        }
    
    except Exception as e:
        logger.error(f"Error creating fine-tuning job: {str(e)}")
        raise ValueError(f"Failed to create fine-tuning job: {str(e)}")


def get_fine_tuning_job_list() -> List[Dict[str, Any]]:
    """Get a fine-tuning job using Google Vertex AI."""
    tuning_jobs = sft.SupervisedTuningJob.list()
    return [job.to_dict() for job in tuning_jobs] if tuning_jobs else []

def get_one_fine_tuning_job(job_name: str) -> Dict[str, Any]:
    """Get a fine-tuning job using Google Vertex AI."""
    tuning_job = sft.SupervisedTuningJob(job_name)
    return tuning_job.to_dict() if tuning_job else {}

def cancel_fine_tuning_job(job_name: str) -> str:
    """Cancel a fine-tuning job using Google Vertex AI."""
    tuning_job = sft.SupervisedTuningJob(job_name)
    tuning_job.cancel()
    return f"Fine-tuning job '{job_name}' cancelled successfully."


