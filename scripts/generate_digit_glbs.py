"""
Generate individual 3D digit/symbol GLB files from BubblegumSans font.
Run with: blender --background --python scripts/generate_digit_glbs.py
"""
import bpy
import os
import sys

FONT_PATH = os.path.join(os.path.dirname(__file__), "..", "client", "assets", "fonts", "BubblegumSans-Regular.ttf")
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "client", "assets", "models", "digits")

# Characters we need for bonus text: digits, x, +
CHARACTERS = {
    "0": "0", "1": "1", "2": "2", "3": "3", "4": "4",
    "5": "5", "6": "6", "7": "7", "8": "8", "9": "9",
    "x": "x", "plus": "+", "dot": ".",
}

EXTRUDE_DEPTH = 0.15
TEXT_SIZE = 1.0


def clear_scene():
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    # Remove orphan data
    for block in bpy.data.meshes:
        if block.users == 0:
            bpy.data.meshes.remove(block)
    for block in bpy.data.materials:
        if block.users == 0:
            bpy.data.materials.remove(block)
    for block in bpy.data.fonts:
        if block.users == 0:
            bpy.data.fonts.remove(block)


def generate_char_glb(char_label, char_value, font_data, output_dir):
    clear_scene()

    # Create text object
    bpy.ops.object.text_add(location=(0, 0, 0))
    text_obj = bpy.context.active_object
    text_obj.data.body = char_value
    text_obj.data.font = font_data
    text_obj.data.size = TEXT_SIZE
    text_obj.data.extrude = EXTRUDE_DEPTH
    text_obj.data.bevel_depth = 0.02  # slight bevel for cartoonish rounded edges
    text_obj.data.bevel_resolution = 2
    text_obj.data.align_x = 'CENTER'
    text_obj.data.align_y = 'CENTER'

    # Convert text to mesh
    bpy.ops.object.convert(target='MESH')
    mesh_obj = bpy.context.active_object

    # Center origin
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    mesh_obj.location = (0, 0, 0)

    # Add a simple white material (game will override with emissive)
    mat = bpy.data.materials.new(name="DigitMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (1.0, 1.0, 1.0, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.3
    mesh_obj.data.materials.append(mat)

    # Apply transforms
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    # Export as GLB
    output_path = os.path.join(output_dir, f"{char_label}.glb")
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        export_format='GLB',
        use_selection=True,
        export_apply=True,
    )
    print(f"Exported: {output_path}")


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Load font
    font_path = os.path.abspath(FONT_PATH)
    if not os.path.exists(font_path):
        print(f"ERROR: Font not found at {font_path}")
        sys.exit(1)

    font_data = bpy.data.fonts.load(font_path)
    print(f"Loaded font: {font_path}")

    for label, char in CHARACTERS.items():
        print(f"Generating '{char}' as {label}.glb ...")
        generate_char_glb(label, char, font_data, OUTPUT_DIR)

    print(f"\nDone! Generated {len(CHARACTERS)} GLB files in {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
