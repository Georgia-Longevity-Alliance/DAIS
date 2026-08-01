defmodule Web.Knowledge.Review do
  use Ecto.Schema
  import Ecto.Changeset

  schema "contribution_reviews" do
    field :score, :float, default: 0.0
    field :feedback, :string
    field :reward_credits, :float, default: 0.0

    belongs_to :claim, Web.Knowledge.Claim
    belongs_to :reviewer, Web.Accounts.User

    timestamps()
  end

  def changeset(review, attrs) do
    review
    |> cast(attrs, [:score, :feedback, :reward_credits, :claim_id, :reviewer_id])
    |> validate_required([:claim_id, :reviewer_id])
  end
end
