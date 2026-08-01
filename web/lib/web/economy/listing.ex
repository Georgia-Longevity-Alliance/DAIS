defmodule Web.Economy.Listing do
  use Ecto.Schema
  import Ecto.Changeset

  @content_types ~w(knowledge_pack dataset analysis_report computation_access consultation)

  schema "marketplace_listings" do
    field :title, :string
    field :description, :string
    field :price_credits, :float
    field :content_type, :string
    field :active, :boolean, default: true
    field :sales_count, :integer, default: 0
    field :revenue_credits, :float, default: 0.0

    belongs_to :user, Web.Accounts.User
    belongs_to :knowledge_claim, Web.Knowledge.Claim

    timestamps()
  end

  def changeset(listing, attrs) do
    listing
    |> cast(attrs, [:title, :description, :price_credits, :content_type, :active, :user_id, :knowledge_claim_id])
    |> validate_required([:title, :price_credits, :user_id])
    |> validate_inclusion(:content_type, @content_types)
    |> validate_number(:price_credits, greater_than: 0)
  end
end
