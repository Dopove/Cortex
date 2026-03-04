# scrapper/tools/image_processing_tools.py

from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, Dict, Any, Optional
from datetime import datetime
from pathlib import Path

# Make sure you have Pillow installed: pip install Pillow
try:
    from PIL import Image, ImageFilter
except ImportError:
    raise ImportError("Pillow library not found. Please install it with 'pip install Pillow'")

class PhotoToSketchArgs(BaseModel):
    """Arguments for the PhotoToSketchTool."""
    image_path: str = Field(..., description="The absolute file path to the input image that needs to be converted to a sketch.")
    output_path: Optional[str] = Field(None, description="Optional custom file path for saving the output sketch image.")

    # FIX: Allows the tool to accept internal context fields from CrewAI without crashing.
    model_config = ConfigDict(extra="allow")

class PhotoToSketchTool(BaseTool):
    """
    A tool that converts a given fashion photograph into a digital line-art sketch.
    It takes a file path for an image, processes it to create a sketch effect,
    and saves the resulting sketch to the designated 'sketches' directory.
    """
    name: str = "PhotoToSketch"
    description: str = "Converts a fashion photo from a file path into a digital sketch."
    args_schema: Type[BaseModel] = PhotoToSketchArgs

    def _run(self, image_path: str, output_path: Optional[str] = None, **kwargs) -> Dict[str, Any]:
        """
        Executes the photo-to-sketch conversion.
        """
        try:
            input_file = Path(image_path)
            if not input_file.is_file():
                return {"success": False, "error": f"Input image not found or is not a file at path: {image_path}"}

            # Open the image using Pillow
            with Image.open(input_file) as img:
                # Convert the image to grayscale for a classic sketch look
                grayscale_img = img.convert("L")
                
                # Apply a contour filter to find edges, creating a sketch-like effect
                sketch_img = grayscale_img.filter(ImageFilter.CONTOUR)

                # Determine the output path based on the project structure
                if output_path:
                    # Use the provided output path but ensure it's just the filename
                    output_filename = Path(output_path).name
                else:
                    # Create a descriptive, unique filename if none is provided
                    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
                    output_filename = f"sketch_{input_file.stem}_{timestamp}.png"

                # Define the correct output directory from your project plan
                output_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/collected_data/visual/sketches")
                output_dir.mkdir(parents=True, exist_ok=True)
                full_output_path = output_dir / output_filename
                
                # Save the final sketch image
                sketch_img.save(full_output_path)

                print(f"✅ Sketch successfully generated and saved to: {full_output_path}")
                return {
                    "success": True,
                    "input_path": image_path,
                    "output_path": str(full_output_path),
                    "processed_at": datetime.now().isoformat()
                }

        except Exception as e:
            # Catch any other errors during processing (e.g., corrupted image file)
            return {"success": False, "error": f"An unexpected error occurred during image processing: {str(e)}", "input_path": image_path}

