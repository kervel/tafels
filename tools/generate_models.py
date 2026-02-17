#!/usr/bin/env python3
"""Generate improved procedural GLB models for the alpine landscape."""
import struct
import json
import math
import os

ASSETS_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "models")

def pack_vec3(x, y, z):
    return struct.pack('<fff', x, y, z)

def pack_vec4(r, g, b, a):
    return struct.pack('<ffff', r, g, b, a)

def pack_u16(v):
    return struct.pack('<H', v)

def write_glb(filename, positions, normals, colors, indices, name="Model"):
    """Write a complete GLB file."""
    # Build binary buffer
    pos_data = b''.join(pack_vec3(*p) for p in positions)
    norm_data = b''.join(pack_vec3(*n) for n in normals)
    col_data = b''.join(pack_vec4(*c) for c in colors)
    idx_data = b''.join(pack_u16(i) for i in indices)

    # Pad each section to 4-byte alignment
    def pad4(data):
        r = len(data) % 4
        return data + b'\x00' * (4 - r) if r else data

    pos_data = pad4(pos_data)
    norm_data = pad4(norm_data)
    col_data = pad4(col_data)
    idx_data = pad4(idx_data)

    total_bin = pos_data + norm_data + col_data + idx_data
    vert_count = len(positions)
    idx_count = len(indices)

    # Compute bounding box
    min_pos = [min(p[i] for p in positions) for i in range(3)]
    max_pos = [max(p[i] for p in positions) for i in range(3)]

    gltf = {
        "asset": {"version": "2.0", "generator": "3dt-gen-v2"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": name}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "COLOR_0": 2}, "indices": 3}]}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vert_count, "type": "VEC3",
             "min": min_pos, "max": max_pos},
            {"bufferView": 1, "componentType": 5126, "count": vert_count, "type": "VEC3"},
            {"bufferView": 2, "componentType": 5126, "count": vert_count, "type": "VEC4"},
            {"bufferView": 3, "componentType": 5123, "count": idx_count, "type": "SCALAR"},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": len(pos_data), "target": 34962},
            {"buffer": 0, "byteOffset": len(pos_data), "byteLength": len(norm_data), "target": 34962},
            {"buffer": 0, "byteOffset": len(pos_data) + len(norm_data), "byteLength": len(col_data), "target": 34962},
            {"buffer": 0, "byteOffset": len(pos_data) + len(norm_data) + len(col_data), "byteLength": len(idx_data), "target": 34963},
        ],
        "buffers": [{"byteLength": len(total_bin)}],
    }

    json_str = json.dumps(gltf, separators=(',', ':'))
    # Pad JSON to 4-byte alignment
    while len(json_str) % 4 != 0:
        json_str += ' '
    json_bytes = json_str.encode('utf-8')

    # GLB header
    total_length = 12 + 8 + len(json_bytes) + 8 + len(total_bin)
    header = struct.pack('<III', 0x46546C67, 2, total_length)  # magic, version, length
    json_chunk = struct.pack('<II', len(json_bytes), 0x4E4F534A) + json_bytes  # JSON chunk
    bin_chunk = struct.pack('<II', len(total_bin), 0x004E4942) + total_bin  # BIN chunk

    filepath = os.path.join(ASSETS_DIR, filename)
    with open(filepath, 'wb') as f:
        f.write(header + json_chunk + bin_chunk)
    print(f"  Written: {filepath} ({total_length} bytes, {vert_count} verts, {idx_count//3} tris)")


def make_cylinder(cx, cy, cz, radius, height, segments, color, positions, normals, colors, indices):
    """Add a cylinder to the mesh data."""
    base = len(positions)
    # Bottom ring + top ring
    for ring in range(2):
        y = cy + ring * height
        for i in range(segments):
            angle = 2.0 * math.pi * i / segments
            x = cx + math.cos(angle) * radius
            z = cz + math.sin(angle) * radius
            nx = math.cos(angle)
            nz = math.sin(angle)
            positions.append((x, y, z))
            normals.append((nx, 0.0, nz))
            colors.append(color)

    # Side faces
    for i in range(segments):
        i1 = (i + 1) % segments
        b0 = base + i
        b1 = base + i1
        t0 = base + segments + i
        t1 = base + segments + i1
        indices.extend([b0, t0, b1, b1, t0, t1])

    # Top cap
    top_center = len(positions)
    positions.append((cx, cy + height, cz))
    normals.append((0.0, 1.0, 0.0))
    colors.append(color)
    for i in range(segments):
        i1 = (i + 1) % segments
        indices.extend([top_center, base + segments + i, base + segments + i1])


def make_cone(cx, cy, cz, radius, height, segments, color, positions, normals, colors, indices):
    """Add a cone to the mesh data."""
    base = len(positions)
    tip = (cx, cy + height, cz)

    # Base ring
    for i in range(segments):
        angle = 2.0 * math.pi * i / segments
        x = cx + math.cos(angle) * radius
        z = cz + math.sin(angle) * radius
        # Cone normal: mix of outward and upward
        slope = radius / height
        ny = 1.0 / math.sqrt(1 + slope * slope)
        nr = slope * ny
        nx = math.cos(angle) * nr
        nz = math.sin(angle) * nr
        positions.append((x, cy, z))
        normals.append((nx, ny, nz))
        colors.append(color)

    # Tip vertex (one per face for smooth normals)
    tip_base = len(positions)
    for i in range(segments):
        angle = 2.0 * math.pi * (i + 0.5) / segments
        slope = radius / height
        ny = 1.0 / math.sqrt(1 + slope * slope)
        nr = slope * ny
        positions.append(tip)
        normals.append((math.cos(angle) * nr, ny, math.sin(angle) * nr))
        colors.append(color)

    # Side faces
    for i in range(segments):
        i1 = (i + 1) % segments
        indices.extend([base + i, tip_base + i, base + i1])


def generate_conifer_tree():
    """Generate a layered conifer tree with trunk and foliage tiers."""
    positions, normals, colors, indices = [], [], [], []

    trunk_color = (0.35, 0.22, 0.12, 1.0)
    # Foliage colors - varying greens
    foliage_dark = (0.10, 0.30, 0.08, 1.0)
    foliage_mid = (0.15, 0.40, 0.10, 1.0)
    foliage_light = (0.20, 0.48, 0.14, 1.0)

    # Trunk
    make_cylinder(0, 0, 0, 0.08, 2.2, 8, trunk_color, positions, normals, colors, indices)

    # 4 tiers of foliage cones, getting smaller toward top
    tiers = [
        (0.7, 0.7, 0.55, foliage_dark),    # base_y, height, radius, color
        (1.1, 0.65, 0.45, foliage_mid),
        (1.5, 0.55, 0.35, foliage_light),
        (1.85, 0.45, 0.25, foliage_mid),
    ]
    for base_y, h, r, col in tiers:
        make_cone(0, base_y, 0, r, h, 10, col, positions, normals, colors, indices)

    write_glb("conifer_tree.glb", positions, normals, colors, indices, "ConiferTree")


def generate_alpine_shrub():
    """Generate a multi-sphere bush cluster."""
    positions, normals, colors, indices = [], [], [], []

    bush_colors = [
        (0.12, 0.32, 0.08, 1.0),
        (0.16, 0.38, 0.10, 1.0),
        (0.14, 0.35, 0.12, 1.0),
    ]

    # Generate several overlapping spheroids
    clusters = [
        (0.0,  0.18, 0.0,  0.25, 0.20, 0),   # cx, cy, cz, radius, height_scale, color_idx
        (0.12, 0.15, 0.08, 0.18, 0.16, 1),
        (-0.10, 0.14, 0.06, 0.20, 0.17, 2),
        (0.05, 0.20, -0.10, 0.16, 0.18, 0),
        (-0.06, 0.16, -0.05, 0.22, 0.19, 1),
    ]

    for cx, cy, cz, radius, hs, ci in clusters:
        make_spheroid(cx, cy, cz, radius, hs, 8, 6, bush_colors[ci],
                      positions, normals, colors, indices)

    write_glb("alpine_shrub.glb", positions, normals, colors, indices, "AlpineShrub")


def make_spheroid(cx, cy, cz, radius, height_scale, u_segments, v_segments, color,
                  positions, normals, colors, indices):
    """Add an oblate spheroid (flattened sphere) to the mesh data."""
    base = len(positions)

    for v in range(v_segments + 1):
        phi = math.pi * v / v_segments
        for u in range(u_segments + 1):
            theta = 2.0 * math.pi * u / u_segments
            nx = math.sin(phi) * math.cos(theta)
            ny = math.cos(phi)
            nz = math.sin(phi) * math.sin(theta)
            x = cx + nx * radius
            y = cy + ny * height_scale
            z = cz + nz * radius
            positions.append((x, y, z))
            normals.append((nx, ny, nz))
            colors.append(color)

    # Indices
    for v in range(v_segments):
        for u in range(u_segments):
            a = base + v * (u_segments + 1) + u
            b = a + 1
            c = a + u_segments + 1
            d = c + 1
            indices.extend([a, c, b, b, c, d])


def generate_rock_outcrop():
    """Generate a rocky outcrop model."""
    positions, normals, colors, indices = [], [], [], []

    rock_colors = [
        (0.45, 0.43, 0.40, 1.0),
        (0.50, 0.48, 0.44, 1.0),
        (0.40, 0.38, 0.35, 1.0),
    ]

    import random
    random.seed(777)

    # Main rock body - deformed sphere
    base = len(positions)
    u_seg, v_seg = 10, 8
    for v in range(v_seg + 1):
        phi = math.pi * v / v_seg
        for u in range(u_seg + 1):
            theta = 2.0 * math.pi * u / u_seg
            nx = math.sin(phi) * math.cos(theta)
            ny = math.cos(phi)
            nz = math.sin(phi) * math.sin(theta)

            # Deform the sphere to look rocky
            noise = 0.7 + 0.3 * math.sin(theta * 3) * math.cos(phi * 2)
            noise += 0.15 * math.sin(theta * 7 + 1.3) * math.cos(phi * 5 + 0.7)

            rx = 0.5 * noise
            ry = 0.35 * noise
            rz = 0.45 * noise

            x = nx * rx
            y = ny * ry + 0.2  # lift above ground
            z = nz * rz

            positions.append((x, y, z))
            normals.append((nx, ny, nz))

            # Color variation based on height
            ci = 0 if y < 0.15 else (1 if y < 0.30 else 2)
            colors.append(rock_colors[ci])

    for v in range(v_seg):
        for u in range(u_seg):
            a = base + v * (u_seg + 1) + u
            b = a + 1
            c = a + u_seg + 1
            d = c + 1
            indices.extend([a, c, b, b, c, d])

    # Add a smaller rock beside main one
    base2 = len(positions)
    for v in range(v_seg + 1):
        phi = math.pi * v / v_seg
        for u in range(u_seg + 1):
            theta = 2.0 * math.pi * u / u_seg
            nx = math.sin(phi) * math.cos(theta)
            ny = math.cos(phi)
            nz = math.sin(phi) * math.sin(theta)

            noise = 0.7 + 0.25 * math.sin(theta * 4 + 2.0) * math.cos(phi * 3)
            rx = 0.3 * noise
            ry = 0.2 * noise
            rz = 0.25 * noise

            x = 0.35 + nx * rx
            y = ny * ry + 0.12
            z = 0.15 + nz * rz

            positions.append((x, y, z))
            normals.append((nx, ny, nz))
            ci = 0 if y < 0.1 else 1
            colors.append(rock_colors[ci])

    for v in range(v_seg):
        for u in range(u_seg):
            a = base2 + v * (u_seg + 1) + u
            b = a + 1
            c = a + u_seg + 1
            d = c + 1
            indices.extend([a, c, b, b, c, d])

    write_glb("rock_outcrop.glb", positions, normals, colors, indices, "RockOutcrop")


def generate_dirt_patch():
    """Generate a flat dirt/bare earth patch."""
    positions, normals, colors, indices = [], [], [], []

    import random
    random.seed(555)

    dirt_colors = [
        (0.40, 0.30, 0.18, 1.0),
        (0.45, 0.34, 0.22, 1.0),
        (0.38, 0.28, 0.16, 1.0),
    ]

    # Flat irregular disc shape
    segments = 12
    rings = 4
    base = len(positions)

    # Center vertex
    positions.append((0.0, 0.02, 0.0))
    normals.append((0.0, 1.0, 0.0))
    colors.append(dirt_colors[1])

    for ring in range(1, rings + 1):
        r = ring * 0.3
        for seg in range(segments):
            angle = 2.0 * math.pi * seg / segments
            # Add irregularity
            r_var = r * (0.85 + 0.3 * random.random())
            x = math.cos(angle) * r_var
            z = math.sin(angle) * r_var
            y = 0.01 + 0.02 * random.random()  # very slight height variation
            positions.append((x, y, z))
            normals.append((0.0, 1.0, 0.0))
            colors.append(dirt_colors[random.randint(0, 2)])

    # Triangles from center to first ring
    for i in range(segments):
        i1 = (i + 1) % segments
        indices.extend([0, base + 1 + i, base + 1 + i1])

    # Triangles between rings
    for ring in range(rings - 1):
        for i in range(segments):
            i1 = (i + 1) % segments
            a = base + 1 + ring * segments + i
            b = base + 1 + ring * segments + i1
            c = base + 1 + (ring + 1) * segments + i
            d = base + 1 + (ring + 1) * segments + i1
            indices.extend([a, c, b, b, c, d])

    write_glb("dirt_patch.glb", positions, normals, colors, indices, "DirtPatch")


if __name__ == "__main__":
    os.makedirs(ASSETS_DIR, exist_ok=True)
    print("Generating models...")
    generate_conifer_tree()
    generate_alpine_shrub()
    generate_rock_outcrop()
    generate_dirt_patch()
    print("Done!")
