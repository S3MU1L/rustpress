-- Prevent deletion of the last admin user
-- This constraint prevents a race condition where two concurrent deletions
-- could leave the system with zero admin users.

-- Function to check if deleting a user would remove the last admin
CREATE OR REPLACE FUNCTION check_last_admin_before_delete()
RETURNS TRIGGER AS $$
DECLARE
    admin_count INTEGER;
    is_target_admin BOOLEAN;
BEGIN
    -- Check if the user being deleted/soft-deleted is an admin
    SELECT EXISTS(
        SELECT 1
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = OLD.id AND r.name = 'admin'
    ) INTO is_target_admin;
    
    -- If not an admin, allow the operation
    IF NOT is_target_admin THEN
        -- For DELETE operations, NEW is NULL, so return OLD
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;
    
    -- Count remaining admins (excluding the one being deleted)
    SELECT COUNT(DISTINCT ur.user_id) INTO admin_count
    FROM user_roles ur
    JOIN roles r ON r.id = ur.role_id
    JOIN users u ON u.id = ur.user_id
    WHERE r.name = 'admin' 
        AND ur.user_id != OLD.id
        AND u.deleted_at IS NULL;
    
    -- Prevent deletion if this is the last admin
    IF admin_count = 0 THEN
        RAISE EXCEPTION 'Cannot delete the last admin'
            USING ERRCODE = 'P0001'; -- raise_exception - application-defined exception
    END IF;
    
    -- For DELETE operations, NEW is NULL, so return OLD
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Trigger to enforce the constraint on soft deletes (UPDATE)
CREATE TRIGGER prevent_last_admin_soft_delete
    BEFORE UPDATE ON users
    FOR EACH ROW
    WHEN (OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL)
    EXECUTE FUNCTION check_last_admin_before_delete();

-- Trigger to enforce the constraint on hard deletes (DELETE)
CREATE TRIGGER prevent_last_admin_hard_delete
    BEFORE DELETE ON users
    FOR EACH ROW
    EXECUTE FUNCTION check_last_admin_before_delete();
