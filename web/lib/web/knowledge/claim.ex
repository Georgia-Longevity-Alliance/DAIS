defmodule Web.Knowledge.Claim do
  use Ecto.Schema
  import Ecto.Changeset

  @statuses ~w(proposed supported tested replicated contested refuted superseded open)

  schema "knowledge_claims" do
    field :subject, :string
    field :predicate, :string
    field :object, :string
    field :status, :string, default: "proposed"
    field :confidence, :float, default: 0.5
    field :domain, :string
    field :citation_count, :integer, default: 0
    field :reward_paid, :float, default: 0.0

    belongs_to :user, Web.Accounts.User
    has_many :evidence_items, Web.Knowledge.Evidence
    has_many :reviews, Web.Knowledge.Review

    timestamps()
  end

  def changeset(claim, attrs) do
    claim
    |> cast(attrs, [:subject, :predicate, :object, :status, :confidence, :domain, :user_id])
    |> validate_required([:subject, :predicate, :object, :user_id])
    |> validate_inclusion(:status, @statuses)
  end
end
