# Certus

A project combining a Rust gateway service with a Next.js dashboard.

## Project Structure

```
certus/
├── gateway/           # Rust binary crate for API gateway
├── dashboard/         # Next.js dashboard with TypeScript, TailwindCSS, and shadcn/ui
├── config/            # Configuration files
│   └── default.yaml   # Default configuration
├── docs/              # Documentation
│   ├── proposal.md
│   └── literature_review.md
├── scripts/           # Build and deployment scripts
│   ├── build.sh
│   ├── deploy.sh
│   └── dev.sh
├── .gitignore
└── README.md
```

## Components

### Gateway (`gateway/`)
A Rust-based API gateway service. Built with Cargo.

**Technology Stack:**
- Rust

**Commands:**
```bash
cd gateway
cargo build          # Build in debug mode
cargo build --release # Build in release mode
cargo run            # Run the gateway
```

### Dashboard (`dashboard/`)
A modern web dashboard built with Next.js, featuring TypeScript for type safety, TailwindCSS for styling, and shadcn/ui for UI components.

**Technology Stack:**
- Next.js 16 (App Router)
- TypeScript
- TailwindCSS 4
- shadcn/ui
- React 19

**Commands:**
```bash
cd dashboard
pnpm install         # Install dependencies
pnpm dev             # Start development server
pnpm build           # Build for production
pnpm start           # Start production server
```

## Configuration

Configuration files are located in the `config/` directory. Edit `config/default.yaml` to customize settings for your environment.

## Scripts

Utility scripts are provided in the `scripts/` directory:

- `build.sh` - Build both gateway and dashboard
- `deploy.sh` - Deploy the application
- `dev.sh` - Start development servers

## Getting Started

1. **Gateway Setup:**
   ```bash
   cd gateway
   cargo build
   ```

2. **Dashboard Setup:**
   ```bash
   cd dashboard
   pnpm install
   ```

3. **Run Development Servers:**
   ```bash
   # Terminal 1 - Gateway
   cd gateway && cargo run

   # Terminal 2 - Dashboard
   cd dashboard && pnpm dev
   ```

## Documentation

Additional documentation can be found in the `docs/` directory:
- `proposal.md` - Project proposal
- `literature_review.md` - Literature review and related work

## License

[Add your license here]

