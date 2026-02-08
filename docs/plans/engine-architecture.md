# Rein Engine アーキテクチャ設計書

## 1. 概要

### 1.1 現状

Reinは wgpu 上に構築された6層の3Dレンダリングライブラリである。`Gm<G: Geometry, M: Material>` ジェネリックコンテナパターンにより、任意のジオメトリとマテリアルを型安全に組み合わせる設計が特徴。主にロボティクス可視化（URDF対応）を用途としている。

### 1.2 目指す方向性

レンダリングライブラリから汎用エンジンへ拡張する。以下の機能を段階的に追加する:

- **ECS（Entity Component System）**: hecs による軽量エンティティ管理
- **物理演算**: 剛体シミュレーション + 衝突検出
- **GPUコンピュート**: wgpu コンピュートシェーダーによる物理演算の高速化
- **ゲームループ**: 固定タイムステップ（物理）と可変タイムステップ（描画）の統合

### 1.3 設計原則

1. **シンプルさの維持**: Reinの既存設計思想を尊重し、必要最小限の抽象化
2. **段階的拡張**: Feature flagsで機能を選択的に有効化。最小構成はレンダリングのみ
3. **後方互換性**: 既存APIを破壊しない。`Gm<G,M>` パターンはそのまま使用可能
4. **一方向依存**: 下位レイヤーは上位レイヤーに依存しない

---

## 2. レイヤーアーキテクチャ

### 2.1 全体構成

```
Layer 0: context/       [既存・変更なし]    wgpu Device/Queue ラッパー
Layer 1: core/          [既存・拡張]        GPU プリミティブ + StorageBuffer, ComputePipelineBuilder
Layer 2: compute/       [新規]              コンピュートパス、GPU-CPUデータ転送
Layer 3: renderer/      [既存・変更なし]    カメラ、マテリアル、ジオメトリ、ライト、シャドウ
Layer 4: physics/       [新規]              剛体、衝突検出、ソルバー
Layer 5: ecs/           [新規]              hecs統合、コンポーネント、システム、スケジュール
Layer 6: engine/        [新規]              ゲームループ、App trait
Layer 7: window/        [既存・最小変更]    winit 抽象化
Layer 8: gui/           [既存・変更なし]    テキスト描画
Layer 9: urdf/          [既存・ECS統合追加] URDF ロボットモデル
```

### 2.2 依存関係

```
         urdf (9)     gui (8)
           |             |
           v             v
         ecs (5) <--- engine (6)
         / | \           |
        v  v  v          v
  physics renderer    window (7)
   (4)     (3)
    |       |
    v       v
  compute  core (1)
   (2)      |
    |       |
    +---+---+
        |
    context (0)
```

矢印は「依存する」方向。ecs (5) が renderer (3) と physics (4) の統合レイヤーとなる。

### 2.3 モジュール構成

```
src/
├── context/                         [既存]
│   └── mod.rs
├── core/                            [既存・拡張]
│   ├── mod.rs                       -- StorageBuffer, ComputePipelineBuilder をエクスポート追加
│   ├── buffer.rs                    -- StorageBuffer 追加
│   ├── pipeline.rs                  -- ComputePipelineBuilder 追加
│   ├── instance.rs                  [変更なし]
│   ├── render_states.rs             [変更なし]
│   ├── render_target.rs             [変更なし]
│   ├── texture.rs                   [変更なし]
│   └── vertex.rs                    [変更なし]
├── compute/                         [新規]
│   └── mod.rs                       -- ComputeDispatcher, GPU→CPU readback
├── renderer/                        [変更なし]
│   ├── geometry/
│   ├── material/
│   ├── control/
│   ├── shadow/
│   ├── viewer.rs
│   ├── object.rs
│   ├── light.rs
│   └── culling.rs
├── physics/                         [新規]
│   ├── mod.rs                       -- PhysicsWorld, PhysicsConfig
│   ├── rigid_body.rs                -- RigidBody 状態、積分
│   ├── collider.rs                  -- 衝突形状、サポート関数
│   ├── broadphase.rs                -- Sweep and Prune
│   ├── narrowphase.rs               -- GJK + EPA, SAT
│   ├── contact.rs                   -- ContactManifold, ContactPoint
│   ├── solver.rs                    -- Sequential Impulse ソルバー
│   └── gpu/
│       └── mod.rs                   -- GpuPhysics, GPU Broadphase
├── ecs/                             [新規]
│   ├── mod.rs                       -- 再エクスポート、prelude
│   ├── components/
│   │   ├── mod.rs
│   │   ├── transform.rs             -- Transform, GlobalTransform, Parent, Children
│   │   ├── rendering.rs             -- MeshRenderer, MeshHandle, MaterialHandle
│   │   └── physics.rs               -- RigidBody, Collider コンポーネント
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── transform.rs             -- 階層伝播
│   │   ├── render.rs                -- ECS → レンダリング
│   │   ├── culling.rs               -- 視錐台カリング
│   │   └── physics.rs               -- 物理ブリッジ
│   ├── schedule.rs                  -- システム実行順序
│   └── bridge.rs                    -- Gm<G,M> ↔ ECS 変換
├── engine/                          [新規]
│   └── mod.rs                       -- App trait, run_app, GameLoopConfig
├── window/                          [最小変更]
│   ├── mod.rs                       -- engine統合用の内部リファクタ
│   ├── event.rs                     [変更なし]
│   ├── frame_io.rs                  [変更なし]
│   └── settings.rs                  [変更なし]
├── effect/                          [変更なし]
├── gui/                             [変更なし]
├── urdf/                            [ECS統合追加]
│   ├── mod.rs
│   ├── loader.rs                    [変更なし]
│   └── robot_model.rs               -- spawn_into_world() メソッド追加
├── shaders/                         [既存シェーダー変更なし]
│   ├── common.wgsl
│   ├── pbr.wgsl
│   ├── ...
│   └── compute/                     [新規ディレクトリ]
│       ├── broadphase.wgsl
│       └── integrate.wgsl
└── lib.rs                           -- 新規モジュール宣言追加
```

---

## 3. Feature Flags

```toml
[features]
default = ["window"]
window = ["dep:winit"]
gui = ["dep:glyphon"]

# 新規
ecs = ["dep:hecs"]
physics = ["ecs"]
gpu-physics = ["physics"]
compute = []
engine = ["ecs", "window"]
full = ["engine", "physics", "gpu-physics", "gui"]
```

```toml
[dependencies]
# 新規依存
hecs = { version = "0.10", optional = true }

# 物理演算は外部依存なし（glam を使用）
```

### 段階的採用パス

| 構成 | features | 用途 |
|------|----------|------|
| レンダリングのみ | `default` | 現在と同じ。既存ユーザーに影響なし |
| + コンピュート | `compute` | StorageBuffer, ComputePipelineBuilder のみ |
| + ECS | `ecs` | hecs World + コンポーネント + システム |
| + 物理 | `physics` | CPU物理（剛体+衝突検出） |
| + GPU物理 | `gpu-physics` | 物理のGPUオフロード |
| フルエンジン | `full` | 全機能 |

---

## 4. ECS統合設計（hecs）

### 4.1 コンポーネント

#### Transform

```rust
// src/ecs/components/transform.rs

/// ローカル空間のTransform。位置・回転・スケールを分離して保持。
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn identity() -> Self { ... }
    pub fn from_position(position: Vec3) -> Self { ... }
    pub fn to_matrix(&self) -> Mat4 { ... }
    pub fn from_matrix(mat: Mat4) -> Self { ... }
}

/// ワールド空間のTransform行列。TransformSystem が毎フレーム更新。
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalTransform(pub Mat4);

/// 親エンティティへの参照。
pub struct Parent(pub hecs::Entity);

/// 子エンティティのリスト。
pub struct Children(pub Vec<hecs::Entity>);
```

#### レンダリング

```rust
// src/ecs/components/rendering.rs

/// 共有メッシュリソースへのハンドル。
pub struct MeshHandle(pub Arc<dyn Geometry + Send + Sync>);

/// 共有マテリアルリソースへのハンドル。
pub struct MaterialHandle(pub Arc<dyn MaterialResource>);

/// メッシュ + マテリアルを組み合わせる描画コンポーネント。
/// ECS における Gm<G, M> の等価物。
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub visible: bool,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
}

/// ライトコンポーネント。
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
}

/// カメラコンポーネント。
pub struct CameraComponent {
    pub camera: Camera,
    pub active: bool,
}

/// 視錐台カリング対象マーカー。
pub struct FrustumCullable;

/// カリング通過マーカー（フレーム毎に更新）。
pub struct Visible;
```

#### 物理

```rust
// src/ecs/components/physics.rs

/// 剛体コンポーネント。
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub inertia_tensor: Mat3,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    pub linear_damping: f32,         // デフォルト: 0.01
    pub angular_damping: f32,        // デフォルト: 0.01
    pub restitution: f32,            // 反発係数 (0.0 - 1.0)
    pub friction: f32,               // 摩擦係数 (0.0 - 1.0)
    pub gravity_scale: f32,          // デフォルト: 1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic,    // 力の影響を受ける
    Static,     // 不動
    Kinematic,  // ユーザーが位置を制御
}

/// 衝突判定コンポーネント。
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec3,
    pub is_sensor: bool,  // trueの場合、衝突イベントのみ（物理応答なし）
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, half_height: f32 },
    Cylinder { radius: f32, half_height: f32 },
    ConvexHull { points: Vec<Vec3> },
}
```

### 4.2 MaterialResource trait

既存の `Material` trait を ECS で Arc 共有可能にする拡張 trait:

```rust
// src/ecs/bridge.rs

/// ECS で Arc 共有可能なマテリアル。
/// 既存 Material trait に Send + Sync を要求する。
pub trait MaterialResource: Material + Send + Sync {}

// wgpu の RenderPipeline, BindGroup, Buffer は全て Send + Sync なので、
// 既存マテリアル実装は自動的に条件を満たす。
impl<T: Material + Send + Sync> MaterialResource for T {}
```

### 4.3 システム

#### TransformSystem

```rust
// src/ecs/systems/transform.rs

/// Parent/Children 階層に基づいて GlobalTransform を伝播する。
pub fn transform_system(world: &mut hecs::World) {
    // Phase 1: ルートエンティティ（Parentなし）
    for (_, (transform, global)) in world
        .query_mut::<(&Transform, &mut GlobalTransform)>()
        .without::<&Parent>()
    {
        global.0 = transform.to_matrix();
    }

    // Phase 2: 階層を再帰的に伝播
    // CommandBuffer で Children を走査し、parent の GlobalTransform × 子の Transform を計算
}
```

#### RenderSystem

```rust
// src/ecs/systems/render.rs

/// ECS World からレンダリングデータを抽出してドローコールを発行する。
pub fn render_system(
    world: &hecs::World,
    ctx: &WgpuContext,
    render_pass: &mut wgpu::RenderPass<'_>,
) {
    // 1. アクティブカメラを検索
    let (camera, camera_transform) = find_active_camera(world);

    // 2. ライト収集
    let lights = collect_lights(world);

    // 3. Visible + MeshRenderer を持つエンティティを描画
    for (_, (renderer, global)) in world
        .query::<(&MeshRenderer, &GlobalTransform)>()
        .with::<&Visible>()
        .iter()
    {
        if !renderer.visible { continue; }

        renderer.material.0.update_uniforms(ctx, &camera, global.0, &lights);
        render_pass.set_pipeline(renderer.material.0.pipeline());
        render_pass.set_bind_group(0, renderer.material.0.camera_bind_group(), &[]);
        render_pass.set_bind_group(1, renderer.material.0.model_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, renderer.mesh.0.vertex_buffer().slice());
        renderer.mesh.0.draw(render_pass);
    }
}
```

#### CullingSystem

```rust
// src/ecs/systems/culling.rs

/// 視錐台カリング。Visible タグを付与/除去する。
pub fn culling_system(world: &mut hecs::World) {
    let frustum = build_frustum_from_active_camera(world);
    let mut cmd = hecs::CommandBuffer::new();

    for (entity, (renderer, global)) in world
        .query::<(&MeshRenderer, &GlobalTransform)>()
        .with::<&FrustumCullable>()
        .iter()
    {
        let aabb = compute_world_aabb(&renderer.mesh.0, global.0);
        if frustum.contains_aabb(&aabb) {
            cmd.insert_one(entity, Visible);
        } else {
            cmd.remove_one::<Visible>(entity);
        }
    }

    cmd.run_on(world);
}
```

### 4.4 システム実行スケジュール

```rust
// src/ecs/schedule.rs

/// システム実行ステージ。
pub enum Stage {
    /// 物理前（入力処理、ゲームロジック）
    PrePhysics,
    /// 固定タイムステップ物理
    Physics,
    /// 物理後（Transform伝播）
    PostPhysics,
    /// 描画前（カリング）
    PreRender,
    /// 描画
    Render,
    /// 描画後（GUI、デバッグ）
    PostRender,
}
```

デフォルト実行順序:

```
1. [PrePhysics]  入力処理、ユーザーゲームロジック
2. [Physics]     PhysicsWorld::step() (固定 dt で 0〜N 回実行)
3. [PostPhysics] TransformSystem (階層伝播)
4. [PreRender]   CullingSystem (視錐台カリング)
5. [Render]      RenderSystem (描画)
6. [PostRender]  EffectChain (ポストプロセス), GUI
```

### 4.5 Gm<G,M> → ECS ブリッジ

既存の `Gm<G,M>` パターンからECSエンティティへの変換ユーティリティ:

```rust
// src/ecs/bridge.rs

/// Gm<G, M> を ECS エンティティとしてスポーンする。
pub fn spawn_gm<G, M>(
    world: &mut hecs::World,
    gm: Gm<G, M>,
) -> hecs::Entity
where
    G: Geometry + Send + Sync + 'static,
    M: Material + Send + Sync + 'static,
{
    let transform = Transform::from_matrix(gm.transform);
    let global = GlobalTransform(gm.transform);
    let renderer = MeshRenderer {
        mesh: MeshHandle(Arc::new(gm.geometry)),
        material: MaterialHandle(Arc::new(gm.material)),
        visible: true,
        cast_shadow: true,
        receive_shadow: true,
    };

    world.spawn((transform, global, renderer, FrustumCullable, Visible))
}
```

---

## 5. 物理エンジン設計

### 5.1 PhysicsWorld

```rust
// src/physics/mod.rs

pub struct PhysicsConfig {
    pub gravity: Vec3,               // デフォルト: (0, -9.81, 0)
    pub fixed_timestep: f64,         // デフォルト: 1.0 / 60.0
    pub max_substeps: u32,           // デフォルト: 4
    pub solver_iterations: u32,      // デフォルト: 8
    pub use_gpu: bool,               // デフォルト: false
}

pub struct PhysicsWorld {
    config: PhysicsConfig,
    accumulator: f64,
    broadphase: SweepAndPrune,
    contacts: Vec<ContactManifold>,
    #[cfg(feature = "gpu-physics")]
    gpu_physics: Option<GpuPhysics>,
}

impl PhysicsWorld {
    pub fn new(config: PhysicsConfig) -> Self { ... }

    /// delta_time に基づいて物理シミュレーションを進行する。
    /// 内部で固定タイムステップをアキュムレータで管理。
    pub fn step(&mut self, world: &mut hecs::World, delta_time: f64) {
        self.accumulator += delta_time;
        let mut substeps = 0;

        while self.accumulator >= self.config.fixed_timestep
              && substeps < self.config.max_substeps
        {
            self.fixed_step(world, self.config.fixed_timestep as f32);
            self.accumulator -= self.config.fixed_timestep;
            substeps += 1;
        }
    }
}
```

### 5.2 固定ステップの処理フロー

```
fixed_step(world, dt):
    1. apply_forces()       -- 重力・外力を force_accumulator に加算
    2. integrate_velocities() -- v += (F/m) * dt
    3. broadphase()          -- AABB で衝突候補ペアを抽出
    4. narrowphase()         -- GJK/EPA で正確な衝突判定
    5. solve_contacts()      -- Sequential Impulse で衝突応答 (N回反復)
    6. integrate_positions() -- p += v * dt
    7. sync_transforms()     -- RigidBody の位置 → Transform に同期
    8. clear_forces()        -- force/torque accumulator をリセット
```

### 5.3 Broadphase: Sweep and Prune

```rust
// src/physics/broadphase.rs

/// O(n log n) の衝突候補ペア検出。
/// 各軸のAABB端点をソートし、3軸全てで重なるペアを報告する。
pub struct SweepAndPrune {
    endpoints: [Vec<Endpoint>; 3],  // X, Y, Z 軸
}

struct Endpoint {
    value: f32,
    entity: hecs::Entity,
    is_min: bool,
}

impl SweepAndPrune {
    /// ECS World から Collider + GlobalTransform を読み取り、
    /// AABBが重なるペアを返す。
    pub fn find_pairs(&mut self, world: &hecs::World) -> Vec<(hecs::Entity, hecs::Entity)> {
        // 1. Collider + GlobalTransform からワールド空間 AABB を計算
        // 2. 端点リストを更新（インクリメンタルソートで高速）
        // 3. 3軸全てで重なるペアを返す
    }
}
```

### 5.4 Narrowphase

#### GJK (Gilbert-Johnson-Keerthi)

凸形状の交差判定。サポート関数を使って Minkowski差 上で原点を含む単体を探す。

```rust
// src/physics/narrowphase.rs

/// 2つの凸形状が交差しているか判定する。
/// 交差している場合、Simplex を返す（EPA への入力）。
pub fn gjk_intersection(
    shape_a: &ColliderShape, transform_a: Mat4,
    shape_b: &ColliderShape, transform_b: Mat4,
) -> Option<Simplex> { ... }
```

#### EPA (Expanding Polytope Algorithm)

GJK で交差が検出された後、貫通深度と衝突法線を計算する。

```rust
/// 貫通深度と衝突法線を計算する。
pub fn epa_penetration(
    simplex: &Simplex,
    shape_a: &ColliderShape, transform_a: Mat4,
    shape_b: &ColliderShape, transform_b: Mat4,
) -> ContactManifold { ... }
```

#### SAT (Separating Axis Theorem)

ボックス同士の衝突に最適化された高速アルゴリズム。

```rust
/// ボックス同士の高速衝突判定。
pub fn sat_box_box(
    half_a: Vec3, transform_a: Mat4,
    half_b: Vec3, transform_b: Mat4,
) -> Option<ContactManifold> { ... }
```

### 5.5 接触ソルバー: Sequential Impulse

```rust
// src/physics/contact.rs

pub struct ContactManifold {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
    pub contacts: Vec<ContactPoint>,
    pub normal: Vec3,       // A から B への方向
}

pub struct ContactPoint {
    pub position: Vec3,     // ワールド座標
    pub penetration: f32,
    // Warm starting 用（前フレームのインパルスを保持）
    pub accumulated_normal_impulse: f32,
    pub accumulated_tangent_impulse: [f32; 2],
}
```

```rust
// src/physics/solver.rs

/// Sequential Impulse ソルバー。
/// config.solver_iterations 回の反復で収束させる。
pub fn solve_contacts(
    contacts: &mut [ContactManifold],
    world: &mut hecs::World,
    dt: f32,
) {
    for contact in contacts.iter_mut() {
        for point in &mut contact.contacts {
            // 1. 接触点での相対速度を計算
            // 2. 法線インパルスを計算（Baumgarte 安定化で貫通を補正）
            // 3. インパルスをクランプ (>= 0)
            // 4. 両剛体にインパルスを適用
            // 5. 摩擦インパルスを計算（Coulomb 摩擦モデル）
            // 6. 摩擦コーン内にクランプ
            // 7. 摩擦インパルスを適用
        }
    }
}
```

### 5.6 衝突形状のサポート関数

GJK/EPA で使用するサポート関数（形状上の最遠点を返す）:

```rust
// src/physics/collider.rs

impl ColliderShape {
    /// 指定方向における形状上の最遠点を返す。
    pub fn support(&self, direction: Vec3, transform: Mat4) -> Vec3 {
        match self {
            ColliderShape::Sphere { radius } => {
                let center = transform.transform_point3(Vec3::ZERO);
                center + direction.normalize() * radius
            }
            ColliderShape::Box { half_extents } => {
                // 8頂点から direction との内積が最大のものを返す
            }
            ColliderShape::Capsule { radius, half_height } => {
                // 上下の球体中心から最遠のものを選択し、radius を加算
            }
            // ...
        }
    }
}
```

---

## 6. GPUコンピュート設計

### 6.1 StorageBuffer

```rust
// src/core/buffer.rs に追加

/// GPU ストレージバッファ。コンピュートシェーダーの読み書きに使用。
pub struct StorageBuffer {
    buffer: wgpu::Buffer,
    size: u64,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl StorageBuffer {
    /// 新しいストレージバッファを作成する。
    pub fn new(ctx: &WgpuContext, size: u64, read_only: bool) -> Self {
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("storage_buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE
                 | wgpu::BufferUsages::COPY_DST
                 | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let binding_type = if read_only {
            wgpu::BufferBindingType::Storage { read_only: true }
        } else {
            wgpu::BufferBindingType::Storage { read_only: false }
        };

        // BindGroupLayout と BindGroup を生成
        // ...
    }

    /// 型付きデータからバッファを作成する。
    pub fn from_data<T: bytemuck::Pod>(ctx: &WgpuContext, data: &[T]) -> Self { ... }

    /// CPU → GPU: データを書き込む。
    pub fn write<T: bytemuck::Pod>(&self, ctx: &WgpuContext, data: &[T]) { ... }

    /// GPU → CPU: データを読み戻す（非同期）。
    pub async fn read_back<T: bytemuck::Pod>(&self, ctx: &WgpuContext) -> Vec<T> { ... }
}
```

### 6.2 ComputePipelineBuilder

既存の `PipelineBuilder` と同じビルダーパターンで、コンピュートパイプラインを構築する。

```rust
// src/core/pipeline.rs に追加

pub struct ComputePipelineBuilder<'a> {
    ctx: &'a WgpuContext,
    label: Option<&'a str>,
    shader_source: Option<&'a str>,
    entry_point: &'a str,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(ctx: &'a WgpuContext) -> Self { ... }
    pub fn label(mut self, label: &'a str) -> Self { ... }
    pub fn shader(mut self, source: &'a str) -> Self { ... }
    pub fn entry_point(mut self, entry: &'a str) -> Self { ... }
    pub fn bind_group_layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self { ... }

    pub fn build(self) -> anyhow::Result<wgpu::ComputePipeline> {
        let shader_module = self.ctx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: self.label,
                source: wgpu::ShaderSource::Wgsl(self.shader_source.unwrap().into()),
            }
        );

        let pipeline_layout = self.ctx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: self.label,
                bind_group_layouts: &self.bind_group_layouts,
                immediate_size: 0,
            }
        );

        Ok(self.ctx.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: self.label,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some(self.entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            }
        ))
    }
}
```

### 6.3 GPU物理オフロード戦略

| 処理 | CPU/GPU | 理由 |
|------|---------|------|
| 速度積分 | GPU | 並列性が高い（各ボディ独立） |
| 位置積分 | GPU | 同上 |
| Broadphase AABB | GPU | 大量ペア比較を並列化 |
| Narrowphase (GJK/EPA) | CPU | 分岐が多く、GPUに不向き |
| 接触ソルバー | CPU | Sequential Impulse は本質的に逐次処理 |

**閾値**: ボディ数 256 以上で GPU に切替（GPU ディスパッチのオーバーヘッドを考慮）。

### 6.4 GPU Broadphase コンピュートシェーダー

```wgsl
// src/shaders/compute/broadphase.wgsl

struct AABB {
    min: vec3<f32>,
    entity_id: u32,
    max: vec3<f32>,
    _padding: u32,
};

struct CollisionPair {
    entity_a: u32,
    entity_b: u32,
};

@group(0) @binding(0) var<storage, read> aabbs: array<AABB>;
@group(0) @binding(1) var<storage, read_write> pairs: array<CollisionPair>;
@group(0) @binding(2) var<storage, read_write> pair_count: atomic<u32>;
@group(1) @binding(0) var<uniform> params: vec4<u32>;  // x = num_bodies

@compute @workgroup_size(64)
fn cs_broadphase(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let count = params.x;
    if (i >= count) { return; }

    let a = aabbs[i];
    for (var j = i + 1u; j < count; j++) {
        let b = aabbs[j];
        if (a.min.x <= b.max.x && a.max.x >= b.min.x
         && a.min.y <= b.max.y && a.max.y >= b.min.y
         && a.min.z <= b.max.z && a.max.z >= b.min.z) {
            let idx = atomicAdd(&pair_count, 1u);
            pairs[idx] = CollisionPair(a.entity_id, b.entity_id);
        }
    }
}
```

### 6.5 GPU-CPU 同期

```rust
// src/physics/gpu/mod.rs

pub struct GpuPhysics {
    aabb_buffer: StorageBuffer,
    pair_buffer: StorageBuffer,
    pair_count_buffer: StorageBuffer,
    broadphase_pipeline: wgpu::ComputePipeline,
    integrate_pipeline: wgpu::ComputePipeline,
    staging_buffer: wgpu::Buffer,
}

impl GpuPhysics {
    /// ECS World のデータを GPU バッファにアップロードする。
    pub fn upload(&self, ctx: &WgpuContext, world: &hecs::World) { ... }

    /// Broadphase コンピュートシェーダーをディスパッチする。
    pub fn dispatch_broadphase(&self, ctx: &WgpuContext, body_count: u32) {
        let mut encoder = ctx.create_encoder(Some("physics compute"));
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.broadphase_pipeline);
            pass.set_bind_group(0, &self.data_bind_group, &[]);
            pass.set_bind_group(1, &self.params_bind_group, &[]);
            pass.dispatch_workgroups((body_count + 63) / 64, 1, 1);
        }
        ctx.submit([encoder.finish()]);
    }

    /// GPU から結果を読み戻す。
    pub async fn readback_pairs(&self, ctx: &WgpuContext) -> Vec<CollisionPair> { ... }
}
```

---

## 7. ゲームループ設計

### 7.1 GameLoopConfig

```rust
// src/engine/mod.rs

pub struct GameLoopConfig {
    /// 物理の固定タイムステップ（秒）。デフォルト: 1/60。
    pub fixed_timestep: f64,
    /// 物理のフレーム毎最大サブステップ数。デフォルト: 4。
    pub max_substeps: u32,
}
```

### 7.2 App trait

```rust
/// ECS ベースのゲームアプリケーション trait。
/// 既存の render_loop コールバックの上位 API。
pub trait App {
    /// 初期化。ECS World のセットアップ、リソースロード。
    fn init(&mut self, ctx: &WgpuContext, world: &mut hecs::World);

    /// 毎フレーム呼ばれる（可変タイムステップ）。入力処理、ゲームロジック。
    fn update(&mut self, world: &mut hecs::World, ctx: &SystemContext);

    /// 固定タイムステップで呼ばれる（物理の前）。オプショナル。
    fn fixed_update(&mut self, _world: &mut hecs::World, _dt: f32) {}

    /// レンダリング後に呼ばれる。GUI、デバッグ表示。オプショナル。
    fn post_render(&mut self, _world: &mut hecs::World, _ctx: &SystemContext) {}
}

pub struct SystemContext {
    pub ctx: WgpuContext,
    pub delta_time: f64,
    pub fixed_delta_time: f64,
    pub elapsed_time: f64,
}

/// ゲームアプリケーションを実行する。
pub fn run_app<A: App + 'static>(
    settings: WindowSettings,
    config: GameLoopConfig,
    app: A,
) -> anyhow::Result<()> { ... }
```

### 7.3 フレーム処理フロー

```
RedrawRequested:
    dt = now - last_frame_time

    // 固定タイムステップ物理ループ
    accumulator += dt
    while accumulator >= fixed_dt && substeps < max_substeps:
        app.fixed_update(world, fixed_dt)
        physics_world.fixed_step(world, fixed_dt)  // 5.2 参照
        accumulator -= fixed_dt
        substeps += 1

    // 可変タイムステップ更新
    app.update(world, system_ctx)

    // Transform 階層伝播
    transform_system(world)

    // 視錐台カリング
    culling_system(world)

    // レンダリング
    surface_texture = surface.get_current_texture()
    render_system(world, ctx, render_pass)

    // ポストプロセス
    effect_chain.apply(...)

    // GUI
    app.post_render(world, system_ctx)

    // 表示
    surface_texture.present()
```

### 7.4 既存 render_loop との共存

`engine` feature が無効の場合、既存の `Window::render_loop()` がそのまま使用可能。`engine` feature が有効の場合、`run_app()` が内部的に `render_loop` と同じ winit イベントループを使用するが、ECS World とシステムスケジュールを自動管理する。

```rust
// 既存API（変更なし、engine feature 不要）
window.render_loop(state, |state, frame| {
    // ユーザーが全てを手動制御
    FrameOutput::default()
});

// 新API（engine feature 必要）
run_app(settings, config, my_app);
// → App trait のコールバックが呼ばれる
```

---

## 8. 後方互換性

### 8.1 既存APIの保証

以下のコードは機能追加後も**変更なし**で動作する:

```rust
use rein::{Window, WindowSettings, FrameOutput, Gm, Mesh, ColorMaterial, Camera};

let window = Window::new(WindowSettings::default())?;
window.render_loop((), |state, frame| {
    // 既存パターンそのまま
    FrameOutput::default()
})?;
```

### 8.2 変更影響の一覧

| ファイル | 変更内容 | 既存APIへの影響 |
|---------|---------|--------------|
| `src/core/buffer.rs` | StorageBuffer 追加 | なし（追加のみ） |
| `src/core/pipeline.rs` | ComputePipelineBuilder 追加 | なし（追加のみ） |
| `src/core/mod.rs` | エクスポート追加 | なし |
| `src/lib.rs` | 新モジュール宣言追加 | なし（cfg gated） |
| `src/window/mod.rs` | 内部リファクタ（engine対応） | 公開APIは変更なし |
| `src/urdf/robot_model.rs` | spawn_into_world() 追加 | なし（cfg gated） |

---

## 9. データフロー

### 9.1 フレーム毎の処理フロー

```
ユーザー入力 (Events)
       │
       ▼
  ┌─────────────┐
  │ Input System │  イベントをカメラコントロール・ゲームロジックに配送
  └──────┬──────┘
         │
         ▼
  ┌─────────────────┐
  │ Fixed Update     │  0〜N 回実行（固定 dt）
  │  ├─ apply_forces │   重力・外力
  │  ├─ integrate_v  │   速度積分
  │  ├─ broadphase   │   AABB ペア検出 (CPU or GPU)
  │  ├─ narrowphase  │   GJK/EPA (CPU)
  │  ├─ solve        │   Sequential Impulse (CPU)
  │  ├─ integrate_p  │   位置積分
  │  └─ sync         │   RigidBody → Transform 同期
  └──────┬──────────┘
         │
         ▼
  ┌──────────────────┐
  │ Transform System │  Parent/Children 階層で GlobalTransform を伝播
  └──────┬───────────┘
         │
         ▼
  ┌────────────────┐
  │ Culling System │  Frustum テストで Visible タグを付与/除去
  └──────┬─────────┘
         │
         ▼
  ┌────────────────┐
  │ Render System  │  MeshRenderer + Visible エンティティを描画
  │  ├─ uniforms   │   Camera, Model, Material ユニフォームを更新
  │  ├─ pipeline   │   RenderPipeline をセット
  │  ├─ bind_group │   Group 0/1/2 をバインド
  │  └─ draw       │   VertexBuffer/IndexBuffer で描画
  └──────┬─────────┘
         │
         ▼
  ┌────────────────┐
  │ Effect Chain   │  FXAA, Fog 等のポストプロセス
  └──────┬─────────┘
         │
         ▼
  ┌────────────────┐
  │ GUI / Debug    │  テキスト、デバッグオーバーレイ
  └──────┬─────────┘
         │
         ▼
      Present
```

### 9.2 ECS-Renderer ブリッジ データフロー

```
hecs::World
  │
  ├── Query<(&CameraComponent, &GlobalTransform)>.with::<&Active>()
  │     → Camera (Viewer trait 実装)
  │
  ├── Query<&LightComponent>
  │     → &[&dyn Light] スライス
  │
  └── Query<(&MeshRenderer, &GlobalTransform)>.with::<&Visible>()
        │
        ├── MeshRenderer.material: Arc<dyn MaterialResource>
        │     ├── pipeline()           → Group 0/1/2 のパイプライン
        │     ├── camera_bind_group()  → Group 0 (Camera)
        │     ├── model_bind_group()   → Group 1 (Model)
        │     └── update_uniforms()    → ユニフォームバッファ更新
        │
        ├── MeshRenderer.mesh: Arc<dyn Geometry + Send + Sync>
        │     ├── vertex_buffer()      → &VertexBuffer
        │     ├── index_buffer()       → Option<&IndexBuffer>
        │     └── draw()               → ドローコール発行
        │
        └── GlobalTransform.0: Mat4    → モデル行列
```

---

## 10. API 使用例

### 10.1 ECS シーン

```rust
use rein::ecs::prelude::*;
use rein::engine::{App, GameLoopConfig, SystemContext, run_app};
use rein::*;

struct MyApp;

impl App for MyApp {
    fn init(&mut self, ctx: &WgpuContext, world: &mut hecs::World) {
        // キューブをスポーン
        let mesh = Mesh::cube(ctx, 1.0, [0.8, 0.2, 0.3]);
        let material = PbrMaterial::new(ctx, format)?;
        spawn_gm(world, Gm::new(mesh, material).with_position(0.0, 1.0, 0.0));

        // カメラ
        let camera = Camera::new_perspective(/*...*/);
        world.spawn((
            CameraComponent { camera, active: true },
            Transform::from_position(Vec3::new(5.0, 3.0, 5.0)),
            GlobalTransform::default(),
        ));

        // ライト
        world.spawn((
            LightComponent {
                light_type: LightType::Directional,
                color: Vec3::ONE,
                intensity: 1.0,
            },
            Transform::default(),
            GlobalTransform::default(),
        ));
    }

    fn update(&mut self, world: &mut hecs::World, ctx: &SystemContext) {
        // 全 MeshRenderer エンティティを回転
        for (_, transform) in world.query_mut::<&mut Transform>()
            .with::<&MeshRenderer>()
        {
            transform.rotation *= Quat::from_rotation_y(ctx.delta_time as f32);
        }
    }
}

fn main() -> anyhow::Result<()> {
    run_app(
        WindowSettings::default().title("ECS Demo"),
        GameLoopConfig::default(),
        MyApp,
    )
}
```

### 10.2 物理シーン

```rust
impl App for PhysicsDemo {
    fn init(&mut self, ctx: &WgpuContext, world: &mut hecs::World) {
        // 地面（静的剛体）
        let ground_mesh = Mesh::quad(ctx, 20.0, 20.0, [0.5, 0.5, 0.5]);
        let ground = spawn_gm(world, Gm::new(ground_mesh, material));
        world.insert(ground, (
            RigidBody::new_static(),
            Collider { shape: ColliderShape::Box { half_extents: Vec3::new(10.0, 0.01, 10.0) }, ..default() },
        )).unwrap();

        // 落下するキューブ（動的剛体）
        for i in 0..10 {
            let cube = spawn_gm(world,
                Gm::new(Mesh::cube(ctx, 0.5, [0.2, 0.6, 0.9]), material.clone())
                    .with_position(0.0, 2.0 + i as f32 * 1.2, 0.0)
            );
            world.insert(cube, (
                RigidBody::new_dynamic(1.0),  // 質量 1.0 kg
                Collider { shape: ColliderShape::Box { half_extents: Vec3::splat(0.25) }, ..default() },
            )).unwrap();
        }
    }
}
```

### 10.3 URDF + ECS

```rust
impl App for RobotApp {
    fn init(&mut self, ctx: &WgpuContext, world: &mut hecs::World) {
        let robot = RobotModel::from_urdf(ctx, "robot.urdf", format)?;

        // ロボットを ECS エンティティとしてスポーン（親子階層付き）
        let root = robot.spawn_into_world(world, ctx, format);

        // ベースに物理を追加
        world.insert(root, (
            RigidBody::new_kinematic(),
            Collider { shape: ColliderShape::Box { half_extents: Vec3::new(0.2, 0.1, 0.2) }, ..default() },
        )).unwrap();
    }
}
```

---

## 11. 実装フェーズ

| フェーズ | 内容 | 依存 |
|---------|------|------|
| **Phase 1** | core/ 拡張: StorageBuffer, ComputePipelineBuilder | なし |
| **Phase 2** | compute/ モジュール: ディスパッチヘルパー | Phase 1 |
| **Phase 3** | ecs/ 基盤: コンポーネント, TransformSystem, ブリッジ | なし |
| **Phase 4** | ecs/ レンダリング: RenderSystem, CullingSystem | Phase 3 |
| **Phase 5** | physics/ CPU: 剛体, Broadphase, Narrowphase, ソルバー | Phase 3 |
| **Phase 6** | engine/: App trait, ゲームループ, タイムステップ | Phase 4, 5 |
| **Phase 7** | physics/ GPU: コンピュートシェーダー, GpuPhysics | Phase 2, 5 |
| **Phase 8** | urdf/ ECS統合: spawn_into_world() | Phase 3 |
