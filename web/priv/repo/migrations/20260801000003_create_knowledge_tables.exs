defmodule Web.Repo.Migrations.CreateKnowledgeTables do
  use Ecto.Migration

  def change do
    create table(:knowledge_claims) do
      add :subject, :string, null: false
      add :predicate, :string, null: false
      add :object, :string, null: false
      add :status, :string, default: "proposed"
      add :confidence, :float, default: 0.5
      add :domain, :string
      add :citation_count, :integer, default: 0
      add :reward_paid, :float, default: 0.0
      add :user_id, references(:users), null: false

      timestamps()
    end

    create index(:knowledge_claims, [:status])
    create index(:knowledge_claims, [:domain])
    create index(:knowledge_claims, [:user_id])

    create table(:sources) do
      add :title, :string, null: false
      add :source_type, :string, default: "other"
      add :url, :string
      add :doi, :string
      add :pmid, :string
      add :description, :string
      add :user_id, references(:users), null: false

      timestamps()
    end

    create index(:sources, [:doi])
    create index(:sources, [:pmid])

    create table(:evidence_items) do
      add :evidence_type, :string, default: "supporting"
      add :strength, :float, default: 0.5
      add :description, :string
      add :claim_id, references(:knowledge_claims), null: false
      add :source_id, references(:sources)
      add :user_id, references(:users), null: false

      timestamps()
    end

    create table(:contribution_reviews) do
      add :score, :float, default: 0.0
      add :feedback, :string
      add :reward_credits, :float, default: 0.0
      add :claim_id, references(:knowledge_claims), null: false
      add :reviewer_id, references(:users), null: false

      timestamps()
    end

    create table(:transactions) do
      add :tx_type, :string, null: false
      add :amount, :float, null: false
      add :balance_after, :float
      add :description, :string
      add :reference_type, :string
      add :reference_id, :integer
      add :user_id, references(:users), null: false

      timestamps()
    end

    create index(:transactions, [:user_id])
    create index(:transactions, [:tx_type])

    create table(:marketplace_listings) do
      add :title, :string, null: false
      add :description, :string
      add :price_credits, :float, null: false
      add :content_type, :string
      add :active, :boolean, default: true
      add :sales_count, :integer, default: 0
      add :revenue_credits, :float, default: 0.0
      add :user_id, references(:users), null: false
      add :knowledge_claim_id, references(:knowledge_claims)

      timestamps()
    end

    create index(:marketplace_listings, [:active])
    create index(:marketplace_listings, [:user_id])
  end
end
