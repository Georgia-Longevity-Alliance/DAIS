defmodule Web.Economy.Engine do
  @moduledoc """
  Central economic engine for the AIS Knowledge Marketplace.

  Rules:
  - Contributors earn credits for verified knowledge contributions
  - Consumers spend credits to access premium knowledge
  - Credits can be purchased if balance is insufficient
  - Central node takes 10% fee on marketplace transactions
  - Reputation score increases with verified contributions
  """

  alias Web.Repo
  alias Web.Accounts.User
  alias Web.Economy.{Transaction, Listing}
  alias Web.Knowledge.Claim

  @reward_base 10.0       # Base credits for a verified claim
  @reward_bonus 5.0       # Bonus for high-quality evidence
  @marketplace_fee 0.10    # 10% central node fee
  @purchase_rate 0.01      # $0.01 = 1 credit (1 credit = 1 cent)

  # ── Rewards ──

  @doc "Award credits to a user for a verified contribution."
  def award_contribution(%User{} = user, %Claim{} = claim, quality_score \\ 1.0) do
    amount = @reward_base * quality_score

    user
    |> add_credits(amount, "reward", "claim:#{claim.id}")
    |> case do
      {:ok, _tx} ->
        # Update claim
        claim
        |> Ecto.Changeset.change(reward_paid: amount)
        |> Repo.update!()

        # Update user stats
        user
        |> Ecto.Changeset.change(
          credits_earned_total: user.credits_earned_total + amount,
          contributions_count: user.contributions_count + 1,
          reputation_score: min(user.reputation_score + quality_score * 0.5, 100.0)
        )
        |> Repo.update!()

        {:ok, amount}

      error -> error
    end
  end

  @doc "Award credits for providing strong evidence."
  def award_evidence(%User{} = user, strength) do
    amount = @reward_bonus * strength
    add_credits(user, amount, "reward", "evidence_bonus")
  end

  # ── Marketplace ──

  @doc "Purchase a marketplace listing."
  def purchase_listing(%User{} = buyer, %Listing{} = listing) do
    seller = Repo.get!(User, listing.user_id)

    if buyer.credits_balance < listing.price_credits do
      {:error, :insufficient_credits, needed: listing.price_credits - buyer.credits_balance}
    else
      fee = listing.price_credits * @marketplace_fee
      seller_share = listing.price_credits - fee

      Repo.transaction(fn ->
        # Debit buyer
        buyer
        |> deduct_credits(listing.price_credits, "spend", "listing:#{listing.id}")
        |> case do
          {:ok, _} -> :ok
          error -> Repo.rollback(error)
        end

        # Credit seller
        seller
        |> add_credits(seller_share, "earn", "sale:#{listing.id}")
        |> case do
          {:ok, _} -> :ok
          error -> Repo.rollback(error)
        end

        # Update listing stats
        listing
        |> Ecto.Changeset.change(
          sales_count: listing.sales_count + 1,
          revenue_credits: listing.revenue_credits + listing.price_credits
        )
        |> Repo.update!()

        # Update buyer stats
        buyer
        |> Ecto.Changeset.change(credits_spent_total: buyer.credits_spent_total + listing.price_credits)
        |> Repo.update!()

        {:ok, listing}
      end)
    end
  end

  @doc "Buy additional credits with external payment (simulated)."
  def purchase_credits(%User{} = user, amount_usd) do
    credits = amount_usd / @purchase_rate
    add_credits(user, credits, "purchase", "fiat:#{amount_usd}")
  end

  # ── Core credit operations ──

  defp add_credits(%User{} = user, amount, tx_type, description) do
    new_balance = user.credits_balance + amount

    user
    |> Ecto.Changeset.change(credits_balance: new_balance)
    |> Repo.update!()

    %Transaction{}
    |> Transaction.changeset(%{
      user_id: user.id,
      tx_type: tx_type,
      amount: amount,
      balance_after: new_balance,
      description: description
    })
    |> Repo.insert()
  end

  defp deduct_credits(%User{} = user, amount, tx_type, description) do
    if user.credits_balance < amount do
      {:error, :insufficient_credits}
    else
      new_balance = user.credits_balance - amount

      user
      |> Ecto.Changeset.change(credits_balance: new_balance)
      |> Repo.update!()

      %Transaction{}
      |> Transaction.changeset(%{
        user_id: user.id,
        tx_type: tx_type,
        amount: -amount,
        balance_after: new_balance,
        description: description
      })
      |> Repo.insert()
    end
  end

  # ── Queries ──

  @doc "Get top contributors by reputation."
  def top_contributors(limit \\ 10) do
    import Ecto.Query
    Repo.all(
      from u in User,
        where: u.contributions_count > 0,
        order_by: [desc: u.reputation_score],
        limit: ^limit,
        select: %{name: u.name, email: u.email, reputation: u.reputation_score, earned: u.credits_earned_total, contributions: u.contributions_count}
    )
  end

  @doc "Get economy stats."
  def stats do
    import Ecto.Query

    total_claims = Repo.aggregate(from(c in Claim), :count) || 0
    total_users = Repo.aggregate(from(u in User), :count) || 0
    total_credits = Repo.aggregate(from(u in User), :sum, :credits_balance) || 0.0
    total_listings = Repo.aggregate(from(l in Listing, where: l.active), :count) || 0
    total_sales = Repo.aggregate(from(l in Listing), :sum, :sales_count) || 0

    %{
      total_claims: total_claims,
      total_users: total_users,
      total_credits_in_circulation: total_credits,
      active_listings: total_listings,
      total_sales: total_sales
    }
  end
end
