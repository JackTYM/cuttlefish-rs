<script setup lang="ts">
/**
 * SafetyApprovalHandler - Global handler for pending approvals
 * 
 * This component should be placed in the root layout to handle
 * pending approval notifications globally across the application.
 * 
 * Usage:
 * <SafetyApprovalHandler />
 */
import type { PendingApprovalEvent } from '~/composables/useWebSocket'

const { pendingApprovals, removePendingApproval } = useWebSocket()
const { approveAction, rejectAction, loading } = useSafetyApi()

const currentAction = ref<PendingApprovalEvent | null>(null)
const showModal = ref(false)

// Watch for new pending approvals
watch(pendingApprovals, (approvals) => {
  if (approvals.length > 0 && !currentAction.value) {
    currentAction.value = approvals[0]
    showModal.value = true
  }
}, { immediate: true })

// Handle approval
const handleApprove = async (actionId: string) => {
  const result = await approveAction(actionId)
  if (result.success) {
    removePendingApproval(actionId)
    showModal.value = false
    currentAction.value = pendingApprovals.value[0] || null
    if (currentAction.value) {
      showModal.value = true
    }
  }
}

// Handle rejection
const handleReject = async (actionId: string) => {
  const result = await rejectAction(actionId)
  if (result.success) {
    removePendingApproval(actionId)
    showModal.value = false
    currentAction.value = pendingApprovals.value[0] || null
    if (currentAction.value) {
      showModal.value = true
    }
  }
}

// Handle modal close
const handleClose = () => {
  showModal.value = false
}

// Convert to the format expected by ApprovalModal
const modalAction = computed(() => {
  if (!currentAction.value) return null
  return {
    id: currentAction.value.id,
    projectId: currentAction.value.projectId,
    actionType: currentAction.value.actionType,
    description: currentAction.value.description,
    path: currentAction.value.path,
    command: currentAction.value.command,
    confidence: currentAction.value.confidence,
    confidenceReasoning: currentAction.value.confidenceReasoning,
    riskFactors: currentAction.value.riskFactors,
    createdAt: currentAction.value.createdAt,
    timeoutSecs: currentAction.value.timeoutSecs,
    hasDiff: currentAction.value.hasDiff,
  }
})
</script>

<template>
  <ApprovalModal
    v-if="modalAction"
    :action="modalAction"
    :visible="showModal"
    :loading="loading"
    @approve="handleApprove"
    @reject="handleReject"
    @close="handleClose"
  />
</template>